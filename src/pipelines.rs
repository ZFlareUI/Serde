use std::collections::HashMap;
use chrono::{DateTime, Utc, Datelike};
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::{Product, InventorySnapshot, Transaction, TransactionType, ProductClass, Currency, Money};
use crate::errors::InventoryResult;

/// Data transformation and analytics pipeline for inventory management
pub struct InventoryPipeline {
    products: Vec<Product>,
    inventory: Vec<InventorySnapshot>,
    transactions: Vec<Transaction>,
}

impl InventoryPipeline {
    /// Create a new pipeline with data
    pub fn new(
        products: Vec<Product>,
        inventory: Vec<InventorySnapshot>,
        transactions: Vec<Transaction>
    ) -> Self {
        Self {
            products,
            inventory,
            transactions,
        }
    }

    /// Filter products by various criteria
    pub fn filter_products<F>(&self, predicate: F) -> Vec<&Product>
    where
        F: Fn(&Product) -> bool,
    {
        self.products.iter().filter(|p| predicate(p)).collect()
    }

    /// Filter products by category
    pub fn filter_by_category(&self, category: &str) -> Vec<&Product> {
        self.filter_products(|p| p.category == category)
    }

    /// Filter products by status
    pub fn filter_by_status(&self, status: &crate::models::ProductStatus) -> Vec<&Product> {
        self.filter_products(|p| &p.status == status)
    }

    /// Filter products by classification
    pub fn filter_by_classification(&self, classification: ProductClass) -> Vec<&Product> {
        self.filter_products(|p| p.classification == Some(classification.clone()))
    }

    /// Filter products by price range
    pub fn filter_by_price_range(&self, min_price: Decimal, max_price: Decimal, currency: Currency) -> Vec<&Product> {
        self.filter_products(|p| {
            p.retail_price.currency == currency &&
            p.retail_price.amount >= min_price &&
            p.retail_price.amount <= max_price
        })
    }

    /// Filter low stock products
    pub fn filter_low_stock_products(&self) -> InventoryResult<Vec<ProductStockInfo>> {
        let mut low_stock = Vec::new();

        for product in &self.products {
            let total_stock = self.get_total_stock_for_product(product.id);
            
            if total_stock <= product.reorder_point {
                low_stock.push(ProductStockInfo {
                    product_id: product.id,
                    sku: product.sku.clone(),
                    name: product.name.clone(),
                    current_stock: total_stock,
                    reorder_point: product.reorder_point,
                    minimum_stock: product.minimum_stock,
                    urgency_level: if total_stock == 0 { UrgencyLevel::Critical }
                    else if total_stock <= product.minimum_stock { UrgencyLevel::High }
                    else { UrgencyLevel::Medium },
                });
            }
        }

        low_stock.sort_by(|a, b| a.urgency_level.cmp(&b.urgency_level));
        Ok(low_stock)
    }

    /// Aggregate inventory value by category
    pub fn aggregate_value_by_category(&self) -> InventoryResult<HashMap<String, CategoryValue>> {
        let mut category_map = HashMap::new();

        for product in &self.products {
            let total_stock = self.get_total_stock_for_product(product.id);
            let total_value = product.unit_cost.amount * Decimal::from(total_stock);

            let entry = category_map.entry(product.category.clone()).or_insert(CategoryValue {
                category: product.category.clone(),
                total_products: 0,
                total_units: 0,
                total_value: Money::new(Decimal::ZERO, product.unit_cost.currency.clone()),
                average_unit_cost: Money::new(Decimal::ZERO, product.unit_cost.currency.clone()),
            });

            entry.total_products += 1;
            entry.total_units += total_stock;
            entry.total_value.amount += total_value;
        }

        // Calculate averages
        for category_value in category_map.values_mut() {
            if category_value.total_products > 0 {
                category_value.average_unit_cost.amount = 
                    category_value.total_value.amount / Decimal::from(category_value.total_products);
            }
        }

        Ok(category_map)
    }

    /// Calculate inventory turnover by category
    pub fn calculate_turnover_by_category(&self, days: u32) -> InventoryResult<HashMap<String, TurnoverMetrics>> {
        let mut category_turnover = HashMap::new();

        for product in &self.products {
            let shipped_units = self.calculate_shipped_units(product.id, days);
            let average_inventory = self.calculate_average_inventory_value(product.id);

            if average_inventory > Decimal::ZERO {
                let cogs = product.unit_cost.amount * Decimal::from(shipped_units);
                // Calculate contribution to category turnover

                let entry = category_turnover.entry(product.category.clone()).or_insert(TurnoverMetrics {
                    category: product.category.clone(),
                    turnover_ratio: 0.0,
                    days_inventory_outstanding: 0.0,
                    total_cogs: Money::new(Decimal::ZERO, product.unit_cost.currency.clone()),
                    average_inventory_value: Money::new(Decimal::ZERO, product.unit_cost.currency.clone()),
                });

                entry.total_cogs.amount += cogs;
                entry.average_inventory_value.amount += average_inventory;
            }
        }

        // Calculate final turnover ratios
        for metrics in category_turnover.values_mut() {
            if metrics.average_inventory_value.amount > Decimal::ZERO {
                let turnover_str = (metrics.total_cogs.amount / metrics.average_inventory_value.amount).to_string();
                metrics.turnover_ratio = turnover_str.parse().unwrap_or(0.0);
                metrics.days_inventory_outstanding = days as f64 / metrics.turnover_ratio;
            }
        }

        Ok(category_turnover)
    }

    /// Generate top selling products report
    pub fn top_selling_products(&self, limit: usize, days: u32) -> Vec<ProductSalesInfo> {
        let mut sales_info = Vec::new();

        for product in &self.products {
            let shipped_units = self.calculate_shipped_units(product.id, days);
            let revenue = product.retail_price.amount * Decimal::from(shipped_units);

            sales_info.push(ProductSalesInfo {
                product_id: product.id,
                sku: product.sku.clone(),
                name: product.name.clone(),
                units_sold: shipped_units,
                revenue: Money::new(revenue, product.retail_price.currency.clone()),
                category: product.category.clone(),
            });
        }

        sales_info.sort_by(|a, b| b.units_sold.cmp(&a.units_sold));
        sales_info.truncate(limit);
        sales_info
    }

    /// Generate slow moving products report
    pub fn slow_moving_products(&self, days: u32, min_days_no_movement: u32) -> Vec<SlowMovingProductInfo> {
        let mut slow_moving = Vec::new();
        let cutoff_date = Utc::now() - chrono::Duration::days(min_days_no_movement as i64);

        for product in &self.products {
            let last_movement = self.get_last_movement_date(product.id);
            let shipped_units = self.calculate_shipped_units(product.id, days);
            let current_stock = self.get_total_stock_for_product(product.id);

            let is_slow_moving = shipped_units == 0 || 
                last_movement.map_or(true, |date| date < cutoff_date);

            if is_slow_moving && current_stock > 0 {
                let holding_value = product.unit_cost.amount * Decimal::from(current_stock);
                
                slow_moving.push(SlowMovingProductInfo {
                    product_id: product.id,
                    sku: product.sku.clone(),
                    name: product.name.clone(),
                    current_stock,
                    units_sold_period: shipped_units,
                    days_since_last_movement: last_movement
                        .map(|date| (Utc::now() - date).num_days().max(0) as u32)
                        .unwrap_or(u32::MAX),
                    holding_value: Money::new(holding_value, product.unit_cost.currency.clone()),
                });
            }
        }

        slow_moving.sort_by(|a, b| b.holding_value.amount.cmp(&a.holding_value.amount));
        slow_moving
    }

    /// Advanced analytics: Demand pattern analysis
    pub fn analyze_demand_patterns(&self, product_id: Uuid) -> InventoryResult<DemandPatternAnalysis> {
        let transactions = self.get_product_shipments(product_id);
        
        if transactions.is_empty() {
            return Ok(DemandPatternAnalysis {
                product_id,
                seasonality_score: 0.0,
                trend_direction: TrendDirection::Stable,
                volatility_coefficient: 0.0,
                demand_clusters: Vec::new(),
                recommended_strategy: RecommendedStrategy::Monitor,
            });
        }

        // Calculate monthly demand
        let monthly_demands = self.calculate_monthly_demands(&transactions)?;
        
        // Analyze seasonality (coefficient of variation)
        let seasonality_score = self.calculate_seasonality_score(&monthly_demands);
        
        // Analyze trend
        let trend_direction = self.analyze_trend(&monthly_demands);
        
        // Calculate volatility
        let volatility_coefficient = self.calculate_volatility(&monthly_demands);
        
        // Identify demand clusters (high/medium/low periods)
        let demand_clusters = self.identify_demand_clusters(&monthly_demands);
        
        // Recommend strategy based on analysis
        let recommended_strategy = self.recommend_strategy(seasonality_score, volatility_coefficient, &trend_direction);

        Ok(DemandPatternAnalysis {
            product_id,
            seasonality_score,
            trend_direction,
            volatility_coefficient,
            demand_clusters,
            recommended_strategy,
        })
    }

    /// Map inventory data to custom format
    pub fn map_inventory_to_custom<T, F>(&self, mapper: F) -> Vec<T>
    where
        F: Fn(&InventorySnapshot, Option<&Product>) -> T,
    {
        let product_map: HashMap<Uuid, &Product> = self.products.iter()
            .map(|p| (p.id, p))
            .collect();

        self.inventory.iter()
            .map(|inv| mapper(inv, product_map.get(&inv.product_id).copied()))
            .collect()
    }

    /// Transform product data with enrichment
    pub fn enrich_product_data(&self) -> Vec<EnrichedProductData> {
        self.products.iter().map(|product| {
            let current_stock = self.get_total_stock_for_product(product.id);
            let total_value = product.unit_cost.amount * Decimal::from(current_stock);
            let shipped_last_30_days = self.calculate_shipped_units(product.id, 30);
            let turnover_30_days = if current_stock > 0 {
                shipped_last_30_days as f64 / current_stock as f64
            } else {
                0.0
            };

            EnrichedProductData {
                product: product.clone(),
                current_stock,
                inventory_value: Money::new(total_value, product.unit_cost.currency.clone()),
                shipped_last_30_days,
                turnover_30_days,
                stock_status: if current_stock == 0 {
                    StockStatus::OutOfStock
                } else if current_stock <= product.minimum_stock {
                    StockStatus::Critical
                } else if current_stock <= product.reorder_point {
                    StockStatus::Low
                } else {
                    StockStatus::Adequate
                },
            }
        }).collect()
    }

    // Helper methods
    fn get_total_stock_for_product(&self, product_id: Uuid) -> u32 {
        self.inventory.iter()
            .filter(|inv| inv.product_id == product_id)
            .map(|inv| inv.quantity_on_hand)
            .sum()
    }

    fn calculate_shipped_units(&self, product_id: Uuid, days: u32) -> u32 {
        let cutoff_date = Utc::now() - chrono::Duration::days(days as i64);
        
        self.transactions.iter()
            .filter(|t| {
                t.product_id == product_id &&
                matches!(t.transaction_type, TransactionType::Shipment) &&
                t.timestamp >= cutoff_date
            })
            .map(|t| t.quantity.abs() as u32)
            .sum()
    }

    fn calculate_average_inventory_value(&self, product_id: Uuid) -> Decimal {
        let snapshots: Vec<&InventorySnapshot> = self.inventory.iter()
            .filter(|inv| inv.product_id == product_id)
            .collect();

        if snapshots.is_empty() {
            return Decimal::ZERO;
        }

        let total_value: Decimal = snapshots.iter()
            .map(|inv| inv.average_cost.amount * Decimal::from(inv.quantity_on_hand))
            .sum();

        total_value / Decimal::from(snapshots.len())
    }

    fn get_last_movement_date(&self, product_id: Uuid) -> Option<DateTime<Utc>> {
        self.transactions.iter()
            .filter(|t| t.product_id == product_id)
            .map(|t| t.timestamp)
            .max()
    }

    fn get_product_shipments(&self, product_id: Uuid) -> Vec<&Transaction> {
        self.transactions.iter()
            .filter(|t| t.product_id == product_id && matches!(t.transaction_type, TransactionType::Shipment))
            .collect()
    }

    fn calculate_monthly_demands(&self, transactions: &[&Transaction]) -> InventoryResult<Vec<u32>> {
        let mut monthly_demands = HashMap::new();

        for transaction in transactions {
            let year_month = (transaction.timestamp.year(), transaction.timestamp.month());
            *monthly_demands.entry(year_month).or_insert(0u32) += transaction.quantity.abs() as u32;
        }

        let mut demands: Vec<u32> = monthly_demands.values().cloned().collect();
        demands.sort();
        Ok(demands)
    }

    fn calculate_seasonality_score(&self, monthly_demands: &[u32]) -> f64 {
        if monthly_demands.len() < 3 {
            return 0.0;
        }

        let mean = monthly_demands.iter().sum::<u32>() as f64 / monthly_demands.len() as f64;
        let variance = monthly_demands.iter()
            .map(|&d| (d as f64 - mean).powi(2))
            .sum::<f64>() / monthly_demands.len() as f64;

        if mean > 0.0 {
            (variance.sqrt() / mean).min(2.0) // Cap at 2.0 for practical purposes
        } else {
            0.0
        }
    }

    fn analyze_trend(&self, monthly_demands: &[u32]) -> TrendDirection {
        if monthly_demands.len() < 3 {
            return TrendDirection::Stable;
        }

        let mid_point = monthly_demands.len() / 2;
        let first_half_avg = monthly_demands[..mid_point].iter().sum::<u32>() as f64 / mid_point as f64;
        let second_half_avg = monthly_demands[mid_point..].iter().sum::<u32>() as f64 / (monthly_demands.len() - mid_point) as f64;

        let change_threshold = first_half_avg * 0.1; // 10% threshold

        if second_half_avg > first_half_avg + change_threshold {
            TrendDirection::Increasing
        } else if second_half_avg < first_half_avg - change_threshold {
            TrendDirection::Decreasing
        } else {
            TrendDirection::Stable
        }
    }

    fn calculate_volatility(&self, monthly_demands: &[u32]) -> f64 {
        self.calculate_seasonality_score(monthly_demands) // Volatility is similar to seasonality
    }

    fn identify_demand_clusters(&self, monthly_demands: &[u32]) -> Vec<DemandCluster> {
        if monthly_demands.is_empty() {
            return Vec::new();
        }

        let mean = monthly_demands.iter().sum::<u32>() as f64 / monthly_demands.len() as f64;
        let std_dev = {
            let variance = monthly_demands.iter()
                .map(|&d| (d as f64 - mean).powi(2))
                .sum::<f64>() / monthly_demands.len() as f64;
            variance.sqrt()
        };

        let high_threshold = mean + std_dev;
        let low_threshold = mean - std_dev;

        let mut clusters = Vec::new();
        
        let high_periods = monthly_demands.iter().filter(|&&d| d as f64 > high_threshold).count();
        let low_periods = monthly_demands.iter().filter(|&&d| (d as f64) < low_threshold).count();
        let medium_periods = monthly_demands.len() - high_periods - low_periods;

        if high_periods > 0 {
            clusters.push(DemandCluster {
                cluster_type: ClusterType::High,
                period_count: high_periods,
                average_demand: monthly_demands.iter()
                    .filter(|&&d| d as f64 > high_threshold)
                    .sum::<u32>() / high_periods as u32,
            });
        }

        if medium_periods > 0 {
            clusters.push(DemandCluster {
                cluster_type: ClusterType::Medium,
                period_count: medium_periods,
                average_demand: monthly_demands.iter()
                    .filter(|&&d| (d as f64) >= low_threshold && (d as f64) <= high_threshold)
                    .sum::<u32>() / medium_periods as u32,
            });
        }

        if low_periods > 0 {
            clusters.push(DemandCluster {
                cluster_type: ClusterType::Low,
                period_count: low_periods,
                average_demand: monthly_demands.iter()
                    .filter(|&&d| (d as f64) < low_threshold)
                    .sum::<u32>() / low_periods as u32,
            });
        }

        clusters
    }

    fn recommend_strategy(&self, seasonality: f64, volatility: f64, trend: &TrendDirection) -> RecommendedStrategy {
        match (seasonality > 0.5, volatility > 0.7, trend) {
            (true, _, _) => RecommendedStrategy::SeasonalPlanning, // High seasonality
            (_, true, _) => RecommendedStrategy::SafetyStockIncrease, // High volatility
            (_, _, TrendDirection::Increasing) => RecommendedStrategy::CapacityIncrease,
            (_, _, TrendDirection::Decreasing) => RecommendedStrategy::InventoryReduction,
            _ => RecommendedStrategy::Monitor,
        }
    }
}

// Supporting data structures

#[derive(Debug, Clone)]
pub struct ProductStockInfo {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub current_stock: u32,
    pub reorder_point: u32,
    pub minimum_stock: u32,
    pub urgency_level: UrgencyLevel,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum UrgencyLevel {
    Critical = 3,
    High = 2,
    Medium = 1,
    Low = 0,
}

#[derive(Debug, Clone)]
pub struct CategoryValue {
    pub category: String,
    pub total_products: u32,
    pub total_units: u32,
    pub total_value: Money,
    pub average_unit_cost: Money,
}

#[derive(Debug, Clone)]
pub struct TurnoverMetrics {
    pub category: String,
    pub turnover_ratio: f64,
    pub days_inventory_outstanding: f64,
    pub total_cogs: Money,
    pub average_inventory_value: Money,
}

#[derive(Debug, Clone)]
pub struct ProductSalesInfo {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub units_sold: u32,
    pub revenue: Money,
    pub category: String,
}

#[derive(Debug, Clone)]
pub struct SlowMovingProductInfo {
    pub product_id: Uuid,
    pub sku: String,
    pub name: String,
    pub current_stock: u32,
    pub units_sold_period: u32,
    pub days_since_last_movement: u32,
    pub holding_value: Money,
}

#[derive(Debug, Clone)]
pub struct DemandPatternAnalysis {
    pub product_id: Uuid,
    pub seasonality_score: f64,
    pub trend_direction: TrendDirection,
    pub volatility_coefficient: f64,
    pub demand_clusters: Vec<DemandCluster>,
    pub recommended_strategy: RecommendedStrategy,
}

#[derive(Debug, Clone)]
pub enum TrendDirection {
    Increasing,
    Decreasing,
    Stable,
}

#[derive(Debug, Clone)]
pub enum RecommendedStrategy {
    Monitor,
    SeasonalPlanning,
    SafetyStockIncrease,
    CapacityIncrease,
    InventoryReduction,
}

#[derive(Debug, Clone)]
pub struct DemandCluster {
    pub cluster_type: ClusterType,
    pub period_count: usize,
    pub average_demand: u32,
}

#[derive(Debug, Clone)]
pub enum ClusterType {
    High,
    Medium,
    Low,
}

#[derive(Debug, Clone)]
pub struct EnrichedProductData {
    pub product: Product,
    pub current_stock: u32,
    pub inventory_value: Money,
    pub shipped_last_30_days: u32,
    pub turnover_30_days: f64,
    pub stock_status: StockStatus,
}

#[derive(Debug, Clone)]
pub enum StockStatus {
    OutOfStock,
    Critical,
    Low,
    Adequate,
    Overstock,
}