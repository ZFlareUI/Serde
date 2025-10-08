use std::collections::HashMap;
use chrono::Datelike;
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::{Product, InventorySnapshot, Transaction, TransactionType, ProductClass};
use crate::errors::{InventoryError, InventoryResult};

/// Comprehensive inventory management system with advanced algorithms
#[derive(Debug, Clone)]
pub struct InventorySystem {
    pub products: HashMap<Uuid, Product>,
    pub inventory: HashMap<(Uuid, Uuid), InventorySnapshot>, // (product_id, location_id) -> snapshot
    pub transactions: Vec<Transaction>,
}

impl InventorySystem {
    /// Create a new inventory system
    pub fn new() -> Self {
        Self {
            products: HashMap::new(),
            inventory: HashMap::new(),
            transactions: Vec::new(),
        }
    }

    /// Add a product to the system
    pub fn add_product(&mut self, product: Product) -> InventoryResult<()> {
        product.validate()?;
        self.products.insert(product.id, product);
        Ok(())
    }

    /// Add inventory snapshot
    pub fn add_inventory(&mut self, snapshot: InventorySnapshot) -> InventoryResult<()> {
        let key = (snapshot.product_id, snapshot.location_id);
        self.inventory.insert(key, snapshot);
        Ok(())
    }

    /// Record a transaction
    pub fn record_transaction(&mut self, transaction: Transaction) -> InventoryResult<()> {
        // Update inventory levels based on transaction
        let key = (transaction.product_id, transaction.location_id);
        
        if let Some(snapshot) = self.inventory.get_mut(&key) {
            match transaction.transaction_type {
                TransactionType::Receipt | TransactionType::Return => {
                    snapshot.quantity_on_hand += transaction.quantity.abs() as u32;
                    snapshot.quantity_available += transaction.quantity.abs() as u32;
                }
                TransactionType::Shipment => {
                    let qty = transaction.quantity.abs() as u32;
                    if snapshot.quantity_on_hand < qty {
                        return Err(InventoryError::validation("Insufficient inventory for shipment"));
                    }
                    snapshot.quantity_on_hand -= qty;
                    snapshot.quantity_available = snapshot.quantity_available.saturating_sub(qty);
                }
                TransactionType::Adjustment => {
                    if transaction.quantity >= 0 {
                        snapshot.quantity_on_hand += transaction.quantity as u32;
                        snapshot.quantity_available += transaction.quantity as u32;
                    } else {
                        let qty = transaction.quantity.abs() as u32;
                        snapshot.quantity_on_hand = snapshot.quantity_on_hand.saturating_sub(qty);
                        snapshot.quantity_available = snapshot.quantity_available.saturating_sub(qty);
                    }
                }
                _ => {}
            }
            snapshot.last_movement = Some(transaction.timestamp);
        }

        self.transactions.push(transaction);
        Ok(())
    }

    /// Calculate reorder recommendations using advanced algorithms
    pub fn calculate_reorder_recommendations(&self) -> InventoryResult<Vec<ReorderRecommendation>> {
        let mut recommendations = Vec::new();

        for (product_id, product) in &self.products {
            // Get current inventory levels across all locations
            let total_inventory = self.get_total_inventory_for_product(*product_id);
            
            if product.needs_reorder(total_inventory) {
                // Calculate demand forecast
                let demand_forecast = self.forecast_demand(*product_id, 90)?; // 90-day forecast
                
                // Calculate safety stock using statistical method
                let safety_stock = self.calculate_safety_stock(*product_id)?;
                
                // Calculate recommended order quantity
                let order_quantity = product.calculate_eoq(
                    demand_forecast.annual_demand, 
                    Decimal::from(50) // $50 ordering cost assumption
                )?;

                recommendations.push(ReorderRecommendation {
                    product_id: *product_id,
                    current_stock: total_inventory,
                    reorder_point: product.reorder_point,
                    recommended_quantity: order_quantity,
                    safety_stock,
                    demand_forecast,
                    urgency: calculate_urgency(total_inventory, product.reorder_point, product.minimum_stock),
                });
            }
        }

        // Sort by urgency (most urgent first)
        recommendations.sort_by(|a, b| b.urgency.partial_cmp(&a.urgency).unwrap());
        Ok(recommendations)
    }

    /// ABC Analysis - classify products by value contribution
    pub fn perform_abc_analysis(&mut self) -> InventoryResult<ABCAnalysisResult> {
        let mut product_values = Vec::new();

        // Calculate annual value for each product
        for (product_id, product) in &self.products {
            let annual_demand = self.calculate_annual_demand(*product_id)?;
            let annual_value = product.unit_cost.amount * Decimal::from(annual_demand);
            
            product_values.push(ProductValue {
                product_id: *product_id,
                annual_demand,
                unit_value: product.unit_cost.amount,
                annual_value,
            });
        }

        // Sort by annual value (descending)
        product_values.sort_by(|a, b| b.annual_value.cmp(&a.annual_value));

        let total_value: Decimal = product_values.iter().map(|pv| pv.annual_value).sum();
        let mut cumulative_value = Decimal::ZERO;
        let mut a_products = Vec::new();
        let mut b_products = Vec::new();
        let mut c_products = Vec::new();

        // Classify products: A (80% of value), B (15% of value), C (5% of value)
        for product_value in product_values {
            cumulative_value += product_value.annual_value;
            let cumulative_percentage = cumulative_value / total_value;

            let classification = if cumulative_percentage <= Decimal::from_f64_retain(0.8).unwrap() {
                a_products.push(product_value.product_id);
                ProductClass::A
            } else if cumulative_percentage <= Decimal::from_f64_retain(0.95).unwrap() {
                b_products.push(product_value.product_id);
                ProductClass::B
            } else {
                c_products.push(product_value.product_id);
                ProductClass::C
            };

            // Update product classification
            if let Some(product) = self.products.get_mut(&product_value.product_id) {
                product.classification = Some(classification);
            }
        }

        Ok(ABCAnalysisResult {
            a_products,
            b_products,
            c_products,
            total_products: self.products.len(),
            total_value,
        })
    }

    /// Calculate demand forecast using moving average with trend adjustment
    pub fn forecast_demand(&self, product_id: Uuid, days: u32) -> InventoryResult<DemandForecast> {
        let transactions = self.get_product_transactions(product_id);
        
        if transactions.is_empty() {
            return Ok(DemandForecast {
                product_id,
                forecast_period_days: days,
                predicted_demand: 0,
                annual_demand: 0,
                confidence_level: 0.0,
                trend_factor: 0.0,
            });
        }

        // Calculate historical demand by week
        let weekly_demands = self.calculate_weekly_demands(&transactions)?;
        
        if weekly_demands.len() < 2 {
            // Not enough data for trend analysis
            let avg_weekly = weekly_demands.first().unwrap_or(&0);
            let predicted_demand = (*avg_weekly as f64 * days as f64 / 7.0) as u32;
            
            return Ok(DemandForecast {
                product_id,
                forecast_period_days: days,
                predicted_demand,
                annual_demand: (*avg_weekly as f64 * 52.0) as u32,
                confidence_level: 0.5,
                trend_factor: 0.0,
            });
        }

        // Calculate trend using linear regression
        let trend_factor = self.calculate_trend(&weekly_demands)?;
        
        // Use exponential smoothing for forecast
        let alpha = 0.3; // Smoothing parameter
        let mut forecast = weekly_demands[0] as f64;
        
        for &demand in &weekly_demands[1..] {
            forecast = alpha * demand as f64 + (1.0 - alpha) * forecast;
        }

        // Apply trend adjustment
        let weeks_ahead = days as f64 / 7.0;
        let trend_adjusted_forecast = forecast + (trend_factor * weeks_ahead);
        let predicted_demand = (trend_adjusted_forecast * weeks_ahead).max(0.0) as u32;
        
        // Calculate confidence based on demand variability
        let confidence_level = self.calculate_forecast_confidence(&weekly_demands);

        Ok(DemandForecast {
            product_id,
            forecast_period_days: days,
            predicted_demand,
            annual_demand: (forecast * 52.0).max(0.0) as u32,
            confidence_level,
            trend_factor,
        })
    }

    /// Calculate safety stock using statistical method
    fn calculate_safety_stock(&self, product_id: Uuid) -> InventoryResult<u32> {
        let transactions = self.get_product_transactions(product_id);
        let weekly_demands = self.calculate_weekly_demands(&transactions)?;
        
        if weekly_demands.len() < 3 {
            // Use rule of thumb: 25% of average demand
            let avg_demand = weekly_demands.iter().sum::<u32>() as f64 / weekly_demands.len() as f64;
            return Ok((avg_demand * 0.25) as u32);
        }

        // Calculate standard deviation of weekly demand
        let mean = weekly_demands.iter().sum::<u32>() as f64 / weekly_demands.len() as f64;
        let variance = weekly_demands.iter()
            .map(|&d| (d as f64 - mean).powi(2))
            .sum::<f64>() / (weekly_demands.len() - 1) as f64;
        let std_dev = variance.sqrt();

        // Safety stock = Z-score (95% service level) * std dev * sqrt(lead time)
        let z_score = 1.645; // 95% service level
        let lead_time_weeks = if let Some(product) = self.products.get(&product_id) {
            product.lead_time_days as f64 / 7.0
        } else {
            2.0 // Default 2 weeks
        };

        let safety_stock = z_score * std_dev * lead_time_weeks.sqrt();
        Ok(safety_stock.max(0.0) as u32)
    }

    /// Helper methods
    fn get_total_inventory_for_product(&self, product_id: Uuid) -> u32 {
        self.inventory.values()
            .filter(|snapshot| snapshot.product_id == product_id)
            .map(|snapshot| snapshot.quantity_on_hand)
            .sum()
    }

    fn get_product_transactions(&self, product_id: Uuid) -> Vec<&Transaction> {
        self.transactions.iter()
            .filter(|t| t.product_id == product_id)
            .collect()
    }

    fn calculate_weekly_demands(&self, transactions: &[&Transaction]) -> InventoryResult<Vec<u32>> {
        let mut weekly_demands = HashMap::new();
        
        for transaction in transactions {
            if matches!(transaction.transaction_type, TransactionType::Shipment) {
                // Calculate year-week using ordinal day
                let year = transaction.timestamp.year();
                let ordinal = transaction.timestamp.ordinal();
                let week = (ordinal - 1) / 7 + 1;
                let year_week = (year, week);
                *weekly_demands.entry(year_week).or_insert(0u32) += transaction.quantity.abs() as u32;
            }
        }

        let mut demands: Vec<u32> = weekly_demands.values().cloned().collect();
        demands.sort();
        Ok(demands)
    }

    fn calculate_trend(&self, weekly_demands: &[u32]) -> InventoryResult<f64> {
        if weekly_demands.len() < 2 {
            return Ok(0.0);
        }

        let n = weekly_demands.len() as f64;
        let x_sum: f64 = (0..weekly_demands.len()).map(|i| i as f64).sum();
        let y_sum: f64 = weekly_demands.iter().sum::<u32>() as f64;
        let xy_sum: f64 = weekly_demands.iter()
            .enumerate()
            .map(|(i, &y)| i as f64 * y as f64)
            .sum();
        let x_sq_sum: f64 = (0..weekly_demands.len()).map(|i| (i as f64).powi(2)).sum();

        // Linear regression slope: (n*Σxy - Σx*Σy) / (n*Σx² - (Σx)²)
        let numerator = n * xy_sum - x_sum * y_sum;
        let denominator = n * x_sq_sum - x_sum.powi(2);

        if denominator.abs() < f64::EPSILON {
            Ok(0.0)
        } else {
            Ok(numerator / denominator)
        }
    }

    fn calculate_forecast_confidence(&self, weekly_demands: &[u32]) -> f32 {
        if weekly_demands.len() < 3 {
            return 0.5;
        }

        let mean = weekly_demands.iter().sum::<u32>() as f64 / weekly_demands.len() as f64;
        let variance = weekly_demands.iter()
            .map(|&d| (d as f64 - mean).powi(2))
            .sum::<f64>() / weekly_demands.len() as f64;
        
        let coefficient_of_variation = if mean > 0.0 { variance.sqrt() / mean } else { 1.0 };
        
        // Higher variability = lower confidence
        (1.0 - coefficient_of_variation.min(1.0)).max(0.1) as f32
    }

    fn calculate_annual_demand(&self, product_id: Uuid) -> InventoryResult<u32> {
        let transactions = self.get_product_transactions(product_id);
        let total_shipped: u32 = transactions.iter()
            .filter(|t| matches!(t.transaction_type, TransactionType::Shipment))
            .map(|t| t.quantity.abs() as u32)
            .sum();

        // Annualize based on data period (assuming we have at least some data)
        if transactions.is_empty() {
            return Ok(0);
        }

        let first_date = transactions.iter().map(|t| t.timestamp).min().unwrap();
        let last_date = transactions.iter().map(|t| t.timestamp).max().unwrap();
        let days_of_data = (last_date - first_date).num_days().max(1) as f64;
        
        Ok((total_shipped as f64 * 365.0 / days_of_data) as u32)
    }
}

impl Default for InventorySystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Reorder recommendation with detailed analysis
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ReorderRecommendation {
    pub product_id: Uuid,
    pub current_stock: u32,
    pub reorder_point: u32,
    pub recommended_quantity: u32,
    pub safety_stock: u32,
    pub demand_forecast: DemandForecast,
    pub urgency: f32, // 0.0 to 1.0, higher is more urgent
}

/// Demand forecasting result with confidence metrics
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DemandForecast {
    pub product_id: Uuid,
    pub forecast_period_days: u32,
    pub predicted_demand: u32,
    pub annual_demand: u32,
    pub confidence_level: f32, // 0.0 to 1.0
    pub trend_factor: f64, // Positive = increasing, negative = decreasing
}

/// ABC Analysis classification result
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ABCAnalysisResult {
    pub a_products: Vec<Uuid>, // High value items (typically 20% of items, 80% of value)
    pub b_products: Vec<Uuid>, // Medium value items
    pub c_products: Vec<Uuid>, // Low value items
    pub total_products: usize,
    pub total_value: Decimal,
}

/// Product value analysis for ABC classification
#[derive(Debug, Clone)]
struct ProductValue {
    pub product_id: Uuid,
    #[allow(dead_code)]
    pub annual_demand: u32,
    #[allow(dead_code)]
    pub unit_value: Decimal,
    pub annual_value: Decimal,
}

/// Calculate urgency score based on stock levels
fn calculate_urgency(current_stock: u32, reorder_point: u32, minimum_stock: u32) -> f32 {
    if current_stock == 0 {
        return 1.0; // Maximum urgency
    }

    if current_stock <= minimum_stock {
        return 0.9; // Very high urgency
    }

    if current_stock <= reorder_point {
        let ratio = (reorder_point - current_stock) as f32 / (reorder_point - minimum_stock).max(1) as f32;
        return 0.5 + (0.4 * ratio); // Medium to high urgency
    }

    0.1 // Low urgency
}

/// Inventory turnover analysis
pub fn calculate_inventory_turnover(
    cost_of_goods_sold: Decimal,
    average_inventory_value: Decimal
) -> InventoryResult<f64> {
    if average_inventory_value == Decimal::ZERO {
        return Err(InventoryError::calculation("Average inventory value cannot be zero"));
    }

    let turnover: f64 = (cost_of_goods_sold / average_inventory_value).to_string().parse()
        .map_err(|_| InventoryError::calculation("Failed to calculate inventory turnover"))?;
    
    Ok(turnover)
}

/// Days of inventory outstanding (DIO) calculation
pub fn calculate_days_inventory_outstanding(
    average_inventory_value: Decimal,
    cost_of_goods_sold: Decimal,
    days_in_period: u32
) -> InventoryResult<f64> {
    if cost_of_goods_sold == Decimal::ZERO {
        return Err(InventoryError::calculation("Cost of goods sold cannot be zero"));
    }

    let daily_cogs = cost_of_goods_sold / Decimal::from(days_in_period);
    let dio: f64 = (average_inventory_value / daily_cogs).to_string().parse()
        .map_err(|_| InventoryError::calculation("Failed to calculate DIO"))?;
    
    Ok(dio)
}