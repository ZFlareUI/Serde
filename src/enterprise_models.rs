use serde::{Deserialize, Serialize};
use std::collections::{HashMap, BTreeMap};
use chrono::{DateTime, Utc};
use uuid::Uuid;
use rust_decimal::Decimal;
use crate::models::Money;
use crate::errors::{InventoryError, InventoryResult};

/// Advanced inventory policy with multiple replenishment strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryPolicy {
    pub id: Uuid,
    pub product_id: Uuid,
    pub policy_type: PolicyType,
    pub min_stock: u32,
    pub max_stock: u32,
    pub target_stock: u32,
    pub review_period_days: u16,
    pub service_level_target: f64, // 0.0 to 1.0
    pub seasonal_factors: HashMap<u8, f64>, // Month -> adjustment factor
    pub supplier_selection_criteria: SupplierCriteria,
    pub lead_time_variability: f64,
    pub demand_uncertainty: f64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PolicyType {
    ContinuousReview, // (s, S) policy
    PeriodicReview,   // (R, S) policy
    JustInTime,       // Lean manufacturing approach
    VendorManaged,    // Supplier-controlled inventory
    TwoEchelon,       // Multi-level supply chain
    EconomicOrderInterval, // Time-based ordering
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierCriteria {
    pub cost_weight: f64,
    pub quality_weight: f64,
    pub delivery_weight: f64,
    pub capacity_weight: f64,
    pub risk_weight: f64,
    pub preferred_suppliers: Vec<Uuid>,
    pub backup_suppliers: Vec<Uuid>,
    pub max_single_source_percentage: f64,
}

/// Advanced forecasting models with ensemble capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForecastModel {
    SimpleMovingAverage {
        periods: usize,
    },
    ExponentialSmoothing {
        alpha: f64,
        beta: Option<f64>, // For trend
        gamma: Option<f64>, // For seasonality
    },
    HoltWinters {
        alpha: f64, // Level
        beta: f64,  // Trend
        gamma: f64, // Seasonality
        seasonal_periods: usize,
        multiplicative: bool,
    },
    LinearRegression {
        features: Vec<String>,
        coefficients: Vec<f64>,
        intercept: f64,
    },
    ARIMA {
        autoregressive_order: usize, // p
        differencing_order: usize,   // d
        moving_average_order: usize, // q
        coefficients: ARIMACoefficients,
    },
    EnsembleModel {
        models: Vec<(ForecastModel, f64)>, // Model and weight
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ARIMACoefficients {
    pub ar_coefficients: Vec<f64>,
    pub ma_coefficients: Vec<f64>,
    pub constant: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForecastAccuracy {
    pub model_id: Uuid,
    pub model_type: String,
    pub mad: f64,  // Mean Absolute Deviation
    pub mape: f64, // Mean Absolute Percentage Error
    pub rmse: f64, // Root Mean Square Error
    pub tracking_signal: f64,
    pub forecast_bias: f64,
    pub evaluation_period: DateRange,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start: DateTime<Utc>,
    pub end: DateTime<Utc>,
}

/// Multi-warehouse network optimization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WarehouseNetwork {
    pub id: Uuid,
    pub name: String,
    pub warehouses: HashMap<Uuid, Warehouse>,
    pub shipping_lanes: Vec<ShippingLane>,
    pub demand_regions: HashMap<String, DemandRegion>,
    pub network_constraints: NetworkConstraints,
    pub optimization_settings: OptimizationSettings,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Warehouse {
    pub id: Uuid,
    pub code: String,
    pub location: GeographicLocation,
    pub capacity_cubic_meters: f64,
    pub capacity_units: u32,
    pub operating_cost_per_unit: Money,
    pub handling_cost_per_transaction: Money,
    pub storage_cost_per_cubic_meter_per_day: Money,
    pub labor_efficiency_factor: f64,
    pub automation_level: AutomationLevel,
    pub certifications: Vec<Certification>,
    pub operating_hours: OperatingSchedule,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeographicLocation {
    pub latitude: f64,
    pub longitude: f64,
    pub address: crate::models::Address,
    pub timezone: String,
    pub customs_zone: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AutomationLevel {
    Manual,
    SemiAutomated,
    FullyAutomated,
    DarkWarehouse, // Fully automated, no human workers
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Certification {
    ISO9001,
    ISO14001,
    FDAApproved,
    GMPCompliant,
    HazmatCertified,
    ColdChainCertified,
    SecurityClearance(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatingSchedule {
    pub hours_per_day: HashMap<chrono::Weekday, (u8, u8)>, // (start_hour, end_hour)
    pub holidays: Vec<DateTime<Utc>>,
    pub peak_seasons: Vec<SeasonalPeriod>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonalPeriod {
    pub name: String,
    pub start_month_day: (u32, u32), // (month, day)
    pub end_month_day: (u32, u32),
    pub capacity_multiplier: f64,
    pub cost_multiplier: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShippingLane {
    pub id: Uuid,
    pub from_warehouse_id: Uuid,
    pub to_warehouse_id: Option<Uuid>, // None for customer delivery
    pub to_region_code: Option<String>,
    pub transportation_mode: TransportationMode,
    pub transit_time_hours: u32,
    pub cost_per_unit: Money,
    pub cost_per_shipment: Money,
    pub capacity_constraints: CapacityConstraints,
    pub service_level: f64, // On-time delivery rate
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransportationMode {
    Ground { vehicle_type: VehicleType },
    Air { service_class: AirServiceClass },
    Ocean { container_type: ContainerType },
    Rail { car_type: RailCarType },
    Intermodal { modes: Vec<TransportationMode> },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VehicleType {
    LTL,        // Less Than Truckload
    FTL,        // Full Truckload
    Parcel,
    LastMile,   // Local delivery
    Refrigerated,
    Hazmat,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AirServiceClass {
    NextDay,
    TwoDay,
    Ground,
    International,
    Freight,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerType {
    TwentyFoot,
    FortyFoot,
    FortyFootHC, // High Cube
    Refrigerated,
    Tank,
    Flatbed,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RailCarType {
    Boxcar,
    Tanker,
    Flatcar,
    Refrigerated,
    Automobile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapacityConstraints {
    pub max_weight_kg: f64,
    pub max_volume_cubic_meters: f64,
    pub max_units: u32,
    pub max_shipments_per_day: u32,
    pub dimensional_constraints: DimensionalConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DimensionalConstraints {
    pub max_length_cm: f64,
    pub max_width_cm: f64,
    pub max_height_cm: f64,
    pub max_single_piece_weight_kg: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemandRegion {
    pub code: String,
    pub name: String,
    pub geographic_bounds: Vec<GeographicLocation>,
    pub population: u64,
    pub economic_indicators: EconomicIndicators,
    pub demand_patterns: RegionalDemandPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EconomicIndicators {
    pub gdp_per_capita: Money,
    pub unemployment_rate: f64,
    pub inflation_rate: f64,
    pub consumer_confidence_index: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionalDemandPatterns {
    pub seasonal_multipliers: HashMap<u8, f64>, // Month -> multiplier
    pub day_of_week_patterns: HashMap<chrono::Weekday, f64>,
    pub cultural_events: Vec<CulturalEvent>,
    pub economic_sensitivity: f64, // How demand responds to economic changes
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalEvent {
    pub name: String,
    pub date: DateTime<Utc>,
    pub impact_factor: f64,
    pub affected_categories: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConstraints {
    pub max_transfer_distance_km: f64,
    pub min_order_quantity_for_transfer: u32,
    pub max_inventory_imbalance_ratio: f64,
    pub emergency_replenishment_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationSettings {
    pub objective: OptimizationObjective,
    pub constraints: Vec<OptimizationConstraint>,
    pub algorithm: OptimizationAlgorithm,
    pub time_horizon_days: u32,
    pub planning_frequency: PlanningFrequency,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationObjective {
    MinimizeTotalCost,
    MaximizeServiceLevel,
    BalanceCostAndService { cost_weight: f64, service_weight: f64 },
    MinimizeInventoryInvestment,
    MaximizeROI,
    MinimizeEnvironmentalImpact,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationConstraint {
    ServiceLevelMinimum(f64),
    BudgetMaximum(Money),
    CapacityLimit { location_id: Uuid, max_units: u32 },
    SingleSourceLimit { supplier_id: Uuid, max_percentage: f64 },
    RegionalCoverage { region: String, max_delivery_time_hours: u32 },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationAlgorithm {
    GeneticAlgorithm {
        population_size: usize,
        generations: usize,
        mutation_rate: f64,
        crossover_rate: f64,
    },
    SimulatedAnnealing {
        initial_temperature: f64,
        cooling_rate: f64,
        min_temperature: f64,
    },
    LinearProgramming,
    MixedIntegerProgramming,
    ParticleSwarmOptimization {
        swarm_size: usize,
        iterations: usize,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlanningFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    EventDriven,
}

/// Customer segmentation for differential service levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSegment {
    pub id: Uuid,
    pub name: String,
    pub classification: CustomerClassification,
    pub revenue_contribution: Decimal,
    pub order_frequency: OrderFrequency,
    pub payment_terms: PaymentTerms,
    pub service_level_agreement: ServiceLevelAgreement,
    pub allocation_priority: u8, // 1 (highest) to 10 (lowest)
    pub discount_tier: DiscountTier,
    pub risk_profile: RiskProfile,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CustomerClassification {
    StrategicAccount,
    KeyAccount,
    StandardAccount,
    SmallAccount,
    ProspectAccount,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderFrequency {
    pub average_orders_per_month: f64,
    pub seasonality_factor: f64,
    pub order_size_variation: f64,
    pub predictability_score: f64, // 0.0 to 1.0
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaymentTerms {
    pub net_days: u16,
    pub early_payment_discount_percentage: f64,
    pub early_payment_days: u16,
    pub credit_limit: Money,
    pub credit_rating: CreditRating,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CreditRating {
    AAA, AA, A, BBB, BB, B, CCC, CC, C, D,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLevelAgreement {
    pub target_fill_rate: f64,
    pub target_delivery_time_hours: u32,
    pub penalty_for_late_delivery: Money,
    pub penalty_for_stockout: Money,
    pub priority_allocation_during_shortage: bool,
    pub dedicated_inventory_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscountTier {
    pub tier_name: String,
    pub volume_discount_schedule: BTreeMap<u32, f64>, // Quantity -> discount percentage
    pub loyalty_discount: f64,
    pub promotional_access: PromotionalAccess,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PromotionalAccess {
    None,
    Standard,
    Premium,
    VIP,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RiskProfile {
    pub payment_risk_score: f64, // 0.0 to 1.0
    pub demand_volatility: f64,
    pub geographic_risk: f64,
    pub industry_risk: f64,
    pub relationship_stability: f64,
    pub mitigation_strategies: Vec<RiskMitigationStrategy>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RiskMitigationStrategy {
    CreditInsurance,
    PaymentGuarantee,
    InventoryConsignment,
    RegularReviews,
    AlternativeSuppliers,
}

/// Quality control and inspection management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityControl {
    pub id: Uuid,
    pub product_id: Uuid,
    pub inspection_plan: InspectionPlan,
    pub quality_standards: QualityStandards,
    pub defect_tracking: DefectTracking,
    pub supplier_quality_metrics: SupplierQualityMetrics,
    pub corrective_actions: Vec<CorrectiveAction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionPlan {
    pub sampling_method: SamplingMethod,
    pub inspection_points: Vec<InspectionPoint>,
    pub accept_quality_level: f64, // AQL percentage
    pub inspection_frequency: String, // Simplified to string for now
    pub required_certifications: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SamplingMethod {
    RandomSampling { sample_size: u32 },
    SystematicSampling { interval: u32 },
    StratifiedSampling { strata: Vec<StrataDefinition> },
    MilStd105E { inspection_level: InspectionLevel },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StrataDefinition {
    pub name: String,
    pub criteria: String,
    pub sample_percentage: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InspectionLevel {
    I, II, III, S1, S2, S3, S4,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InspectionPoint {
    pub id: Uuid,
    pub name: String,
    pub location_in_process: ProcessLocation,
    pub test_methods: Vec<TestMethod>,
    pub acceptance_criteria: AcceptanceCriteria,
    pub documentation_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProcessLocation {
    Receiving,
    InProcess(String),
    PreShipment,
    CustomerReturn,
    PeriodicAudit,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestMethod {
    pub name: String,
    pub procedure_reference: String,
    pub equipment_required: Vec<String>,
    pub skill_level_required: SkillLevel,
    pub test_duration_minutes: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SkillLevel {
    Basic,
    Intermediate,
    Advanced,
    Expert,
    CertifiedOnly,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AcceptanceCriteria {
    pub measurement_type: MeasurementType,
    pub specification_limits: SpecificationLimits,
    pub statistical_control: bool,
    pub process_capability_requirements: Option<ProcessCapability>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MeasurementType {
    Continuous { unit: String },
    Discrete { categories: Vec<String> },
    Binary { pass_fail: bool },
    Count { defects_per_unit: bool },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpecificationLimits {
    pub lower_limit: Option<f64>,
    pub upper_limit: Option<f64>,
    pub target_value: Option<f64>,
    pub tolerance: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCapability {
    pub cp_minimum: f64,  // Process capability
    pub cpk_minimum: f64, // Process capability index
    pub pp_minimum: f64,  // Process performance
    pub ppk_minimum: f64, // Process performance index
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityStandards {
    pub iso_standards: Vec<String>,
    pub industry_standards: Vec<String>,
    pub regulatory_requirements: Vec<RegulatoryRequirement>,
    pub customer_specifications: Vec<CustomerSpecification>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryRequirement {
    pub agency: String,
    pub regulation_number: String,
    pub effective_date: DateTime<Utc>,
    pub compliance_deadline: DateTime<Utc>,
    pub requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerSpecification {
    pub customer_id: Uuid,
    pub specification_document: String,
    pub version: String,
    pub effective_date: DateTime<Utc>,
    pub special_requirements: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectTracking {
    pub defect_categories: Vec<DefectCategory>,
    pub root_cause_analysis: Vec<RootCauseAnalysis>,
    pub trend_analysis: TrendAnalysis,
    pub cost_of_quality: CostOfQuality,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefectCategory {
    pub name: String,
    pub severity: DefectSeverity,
    pub frequency: u32,
    pub cost_impact: Money,
    pub prevention_actions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DefectSeverity {
    Critical,    // Safety or regulatory
    Major,       // Functional failure
    Minor,       // Cosmetic or performance
    Negligible,  // No impact
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCauseAnalysis {
    pub incident_id: Uuid,
    pub defect_description: String,
    pub investigation_method: InvestigationMethod,
    pub root_causes: Vec<RootCause>,
    pub corrective_actions: Vec<String>,
    pub preventive_actions: Vec<String>,
    pub effectiveness_verification: EffectivenessVerification,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InvestigationMethod {
    FishboneDiagram,
    FiveWhys,
    FaultTreeAnalysis,
    FailureModeEffectAnalysis,
    StatisticalAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RootCause {
    pub category: RootCauseCategory,
    pub description: String,
    pub contributing_factors: Vec<String>,
    pub likelihood: f64,
    pub impact: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RootCauseCategory {
    Material,
    Method,
    Machine,
    Manpower,
    Measurement,
    Environment,
    Design,
    Supplier,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessVerification {
    pub verification_method: String,
    pub success_criteria: String,
    pub verification_date: Option<DateTime<Utc>>,
    pub verified_by: String,
    pub effectiveness_score: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub defect_rate_trend: Vec<TrendDataPoint>,
    pub supplier_performance_trend: Vec<SupplierTrendPoint>,
    pub process_capability_trend: Vec<ProcessCapabilityTrend>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendDataPoint {
    pub timestamp: DateTime<Utc>,
    pub defect_rate: f64,
    pub volume: u32,
    pub cost_impact: Money,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierTrendPoint {
    pub supplier_id: Uuid,
    pub timestamp: DateTime<Utc>,
    pub quality_score: f64,
    pub delivery_performance: f64,
    pub cost_competitiveness: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessCapabilityTrend {
    pub process_id: String,
    pub timestamp: DateTime<Utc>,
    pub cp: f64,
    pub cpk: f64,
    pub sigma_level: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostOfQuality {
    pub prevention_costs: Money,
    pub appraisal_costs: Money,
    pub internal_failure_costs: Money,
    pub external_failure_costs: Money,
    pub opportunity_costs: Money,
    pub total_cost: Money,
    pub cost_as_percentage_of_revenue: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierQualityMetrics {
    pub incoming_quality_rate: f64,
    pub supplier_corrective_action_requests: u32,
    pub audit_scores: Vec<SupplierAuditScore>,
    pub certification_status: Vec<CertificationStatus>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SupplierAuditScore {
    pub audit_date: DateTime<Utc>,
    pub auditor: String,
    pub overall_score: f64,
    pub category_scores: HashMap<String, f64>,
    pub findings: Vec<AuditFinding>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditFinding {
    pub severity: FindingSeverity,
    pub category: String,
    pub description: String,
    pub corrective_action_required: bool,
    pub due_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FindingSeverity {
    Observation,
    MinorNonConformance,
    MajorNonConformance,
    CriticalNonConformance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CertificationStatus {
    pub certification_type: String,
    pub status: CertificationState,
    pub expiry_date: Option<DateTime<Utc>>,
    pub certifying_body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CertificationState {
    Valid,
    Expired,
    Suspended,
    UnderReview,
    NotRequired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CorrectiveAction {
    pub id: Uuid,
    pub issue_description: String,
    pub root_cause: String,
    pub action_plan: String,
    pub responsible_party: String,
    pub due_date: DateTime<Utc>,
    pub status: ActionStatus,
    pub verification_method: String,
    pub completion_date: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionStatus {
    Open,
    InProgress,
    PendingVerification,
    Completed,
    Overdue,
    Cancelled,
}

impl InventoryPolicy {
    /// Calculate dynamic safety stock based on policy parameters
    pub fn calculate_dynamic_safety_stock(
        &self,
        demand_std_dev: f64,
        lead_time_days: f64,
    ) -> InventoryResult<u32> {
        // Z-score for service level
        let z_score = match self.service_level_target {
            x if x >= 0.999 => 3.09,
            x if x >= 0.99 => 2.33,
            x if x >= 0.95 => 1.65,
            x if x >= 0.90 => 1.28,
            x if x >= 0.85 => 1.04,
            _ => 0.84, // 80% service level
        };

        // Account for lead time variability
        let lead_time_variance = (lead_time_days * self.lead_time_variability).powi(2);
        let demand_variance = demand_std_dev.powi(2);
        
        // Combined variance
        let total_variance = (lead_time_days * demand_variance) + (demand_variance * lead_time_variance);
        let total_std_dev = total_variance.sqrt();

        let safety_stock = z_score * total_std_dev;
        Ok(safety_stock.round() as u32)
    }

    /// Get seasonal adjustment factor for current month
    pub fn get_seasonal_factor(&self, month: u8) -> f64 {
        self.seasonal_factors.get(&month).copied().unwrap_or(1.0)
    }
}

impl ForecastModel {
    /// Generate forecast for specified periods
    pub fn forecast(&self, historical_data: &[f64], periods: usize) -> InventoryResult<Vec<f64>> {
        match self {
            ForecastModel::SimpleMovingAverage { periods: ma_periods } => {
                if historical_data.len() < *ma_periods {
                    return Err(InventoryError::calculation("Insufficient historical data for moving average"));
                }
                
                let recent_data = &historical_data[historical_data.len() - ma_periods..];
                let average = recent_data.iter().sum::<f64>() / *ma_periods as f64;
                Ok(vec![average; periods])
            }
            
            ForecastModel::ExponentialSmoothing { alpha, beta, gamma } => {
                self.exponential_smoothing_forecast(historical_data, periods, *alpha, *beta, *gamma)
            }
            
            ForecastModel::HoltWinters { alpha, beta, gamma, seasonal_periods, multiplicative } => {
                self.holt_winters_forecast(historical_data, periods, *alpha, *beta, *gamma, *seasonal_periods, *multiplicative)
            }
            
            ForecastModel::LinearRegression { coefficients, intercept, .. } => {
                let mut forecasts = Vec::new();
                let base_period = historical_data.len();
                
                for i in 0..periods {
                    let period = (base_period + i + 1) as f64;
                    let forecast = intercept + coefficients[0] * period;
                    forecasts.push(forecast.max(0.0));
                }
                Ok(forecasts)
            }
            
            ForecastModel::ARIMA { .. } => {
                // Simplified ARIMA implementation
                self.arima_forecast(historical_data, periods)
            }
            
            ForecastModel::EnsembleModel { models } => {
                let mut ensemble_forecasts = vec![0.0; periods];
                let total_weight: f64 = models.iter().map(|(_, weight)| weight).sum();
                
                for (model, weight) in models {
                    let model_forecasts = model.forecast(historical_data, periods)?;
                    for (i, forecast) in model_forecasts.iter().enumerate() {
                        ensemble_forecasts[i] += (forecast * weight) / total_weight;
                    }
                }
                Ok(ensemble_forecasts)
            }
        }
    }

    fn exponential_smoothing_forecast(
        &self,
        data: &[f64],
        periods: usize,
        alpha: f64,
        beta: Option<f64>,
        _gamma: Option<f64>,
    ) -> InventoryResult<Vec<f64>> {
        if data.is_empty() {
            return Err(InventoryError::calculation("No historical data provided"));
        }

        let mut level = data[0];
        let mut trend = if data.len() > 1 { data[1] - data[0] } else { 0.0 };

        // Update level and trend through historical data
        for i in 1..data.len() {
            let new_level = alpha * data[i] + (1.0 - alpha) * (level + trend);
            if let Some(beta_val) = beta {
                trend = beta_val * (new_level - level) + (1.0 - beta_val) * trend;
            }
            level = new_level;
        }

        // Generate forecasts
        let mut forecasts = Vec::new();
        for h in 1..=periods {
            let forecast = if beta.is_some() {
                level + (h as f64) * trend
            } else {
                level
            };
            forecasts.push(forecast.max(0.0));
        }

        Ok(forecasts)
    }

    fn holt_winters_forecast(
        &self,
        data: &[f64],
        periods: usize,
        alpha: f64,
        beta: f64,
        gamma: f64,
        seasonal_periods: usize,
        multiplicative: bool,
    ) -> InventoryResult<Vec<f64>> {
        if data.len() < 2 * seasonal_periods {
            return Err(InventoryError::calculation("Insufficient data for Holt-Winters"));
        }

        let mut level = data[0];
        let mut trend = (data[seasonal_periods] - data[0]) / seasonal_periods as f64;
        let mut seasonal = vec![1.0; seasonal_periods];

        // Initialize seasonal factors
        for i in 0..seasonal_periods {
            if multiplicative {
                seasonal[i] = data[i] / level;
            } else {
                seasonal[i] = data[i] - level;
            }
        }

        // Update parameters through historical data
        for t in seasonal_periods..data.len() {
            let s = t % seasonal_periods;
            
            let new_level = if multiplicative {
                alpha * (data[t] / seasonal[s]) + (1.0 - alpha) * (level + trend)
            } else {
                alpha * (data[t] - seasonal[s]) + (1.0 - alpha) * (level + trend)
            };

            let new_trend = beta * (new_level - level) + (1.0 - beta) * trend;
            
            let new_seasonal = if multiplicative {
                gamma * (data[t] / new_level) + (1.0 - gamma) * seasonal[s]
            } else {
                gamma * (data[t] - new_level) + (1.0 - gamma) * seasonal[s]
            };

            level = new_level;
            trend = new_trend;
            seasonal[s] = new_seasonal;
        }

        // Generate forecasts
        let mut forecasts = Vec::new();
        for h in 1..=periods {
            let s = (data.len() + h - 1) % seasonal_periods;
            let forecast = if multiplicative {
                (level + (h as f64) * trend) * seasonal[s]
            } else {
                level + (h as f64) * trend + seasonal[s]
            };
            forecasts.push(forecast.max(0.0));
        }

        Ok(forecasts)
    }

    fn arima_forecast(&self, data: &[f64], periods: usize) -> InventoryResult<Vec<f64>> {
        if data.len() < 10 {
            return Err(InventoryError::calculation("Insufficient data for ARIMA"));
        }

        // Simplified ARIMA - use last value with small random walk
        let last_value = data[data.len() - 1];
        let _volatility = self.calculate_volatility(data);
        
        let mut forecasts = Vec::new();
        let mut current_forecast = last_value;
        
        for _ in 0..periods {
            // Simple random walk with mean reversion
            current_forecast = current_forecast * 0.95 + last_value * 0.05;
            forecasts.push(current_forecast.max(0.0));
        }

        Ok(forecasts)
    }

    fn calculate_volatility(&self, data: &[f64]) -> f64 {
        if data.len() < 2 {
            return 0.1;
        }

        let mean = data.iter().sum::<f64>() / data.len() as f64;
        let variance = data.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / (data.len() - 1) as f64;
        variance.sqrt()
    }
}

impl QualityControl {
    /// Calculate overall quality score based on multiple metrics
    pub fn calculate_quality_score(&self) -> f64 {
        let defect_score = self.calculate_defect_score();
        let supplier_score = self.supplier_quality_metrics.incoming_quality_rate;
        let process_score = self.calculate_process_capability_score();
        
        // Weighted average
        (defect_score * 0.4 + supplier_score * 0.3 + process_score * 0.3).min(1.0).max(0.0)
    }

    fn calculate_defect_score(&self) -> f64 {
        let total_defects: u32 = self.defect_tracking.defect_categories.iter()
            .map(|cat| cat.frequency)
            .sum();
        
        if total_defects == 0 {
            return 1.0;
        }

        // Weighted by severity
        let weighted_defects: f64 = self.defect_tracking.defect_categories.iter()
            .map(|cat| {
                let severity_weight = match cat.severity {
                    DefectSeverity::Critical => 4.0,
                    DefectSeverity::Major => 3.0,
                    DefectSeverity::Minor => 2.0,
                    DefectSeverity::Negligible => 1.0,
                };
                cat.frequency as f64 * severity_weight
            })
            .sum();

        1.0 - (weighted_defects / (total_defects as f64 * 4.0))
    }

    fn calculate_process_capability_score(&self) -> f64 {
        // Process capability calculation based on quality metrics
        // Uses Six Sigma approach: Cp = (USL - LSL) / (6 * sigma)
        let defect_rate = self.total_defects as f64 / self.total_units as f64;
        let process_sigma = if defect_rate > 0.0 {
            (-defect_rate.ln()).sqrt() / 2.0
        } else {
            6.0 // Perfect process assumed at 6-sigma level
        };
        
        // Convert to capability score (0.0 to 1.0 scale)
        (process_sigma / 6.0).min(1.0).max(0.0)
    }
}