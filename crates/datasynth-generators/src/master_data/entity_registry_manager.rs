//! Entity Registry Manager for coordinated master data generation.
//!
//! This module provides a central manager that coordinates the generation
//! of all master data entities and maintains the entity registry for
//! referential integrity in transaction generation.

use chrono::NaiveDate;
use datasynth_core::models::{
    CustomerPool, Employee, EmployeePool, EntityId, EntityRegistry, EntityType, FixedAsset,
    FixedAssetPool, Material, MaterialPool, Vendor, VendorPool,
};

use super::{
    AssetGenerator, AssetGeneratorConfig, CustomerGenerator, CustomerGeneratorConfig,
    EmployeeGenerator, EmployeeGeneratorConfig, MaterialGenerator, MaterialGeneratorConfig,
    VendorGenerator, VendorGeneratorConfig,
};

/// Configuration for the entity registry manager.
#[derive(Debug, Clone, Default)]
pub struct EntityRegistryManagerConfig {
    /// Vendor generator configuration
    pub vendor_config: VendorGeneratorConfig,
    /// Customer generator configuration
    pub customer_config: CustomerGeneratorConfig,
    /// Material generator configuration
    pub material_config: MaterialGeneratorConfig,
    /// Asset generator configuration
    pub asset_config: AssetGeneratorConfig,
    /// Employee generator configuration
    pub employee_config: EmployeeGeneratorConfig,
}

/// Counts for master data generation.
#[derive(Debug, Clone)]
pub struct MasterDataCounts {
    /// Number of vendors
    pub vendors: usize,
    /// Number of customers
    pub customers: usize,
    /// Number of materials
    pub materials: usize,
    /// Number of fixed assets
    pub assets: usize,
    /// Number of employees (auto-calculated from departments if None)
    pub employees: Option<usize>,
}

impl Default for MasterDataCounts {
    fn default() -> Self {
        Self {
            vendors: 100,
            customers: 200,
            materials: 500,
            assets: 150,
            employees: None,
        }
    }
}

/// Generated master data result.
#[derive(Debug)]
pub struct GeneratedMasterData {
    /// Vendor pool
    pub vendors: VendorPool,
    /// Customer pool
    pub customers: CustomerPool,
    /// Material pool
    pub materials: MaterialPool,
    /// Fixed asset pool
    pub assets: FixedAssetPool,
    /// Employee pool
    pub employees: EmployeePool,
    /// Entity registry with all entities
    pub registry: EntityRegistry,
}

/// Entity Registry Manager for coordinated master data generation.
pub struct EntityRegistryManager {
    // Retained for future use (e.g., reseed after reset, config-driven generation).
    #[allow(dead_code)]
    seed: u64,
    #[allow(dead_code)]
    config: EntityRegistryManagerConfig,
    vendor_generator: VendorGenerator,
    customer_generator: CustomerGenerator,
    material_generator: MaterialGenerator,
    asset_generator: AssetGenerator,
    employee_generator: EmployeeGenerator,
    registry: EntityRegistry,
}

impl EntityRegistryManager {
    /// Create a new entity registry manager.
    pub fn new(seed: u64) -> Self {
        Self::with_config(seed, EntityRegistryManagerConfig::default())
    }

    /// Create a new entity registry manager with custom configuration.
    pub fn with_config(seed: u64, config: EntityRegistryManagerConfig) -> Self {
        Self {
            seed,
            vendor_generator: VendorGenerator::with_config(seed, config.vendor_config.clone()),
            customer_generator: CustomerGenerator::with_config(
                seed + 1,
                config.customer_config.clone(),
            ),
            material_generator: MaterialGenerator::with_config(
                seed + 2,
                config.material_config.clone(),
            ),
            asset_generator: AssetGenerator::with_config(seed + 3, config.asset_config.clone()),
            employee_generator: EmployeeGenerator::with_config(
                seed + 4,
                config.employee_config.clone(),
            ),
            registry: EntityRegistry::new(),
            config,
        }
    }

    /// Generate all master data for a company.
    pub fn generate_master_data(
        &mut self,
        company_code: &str,
        counts: &MasterDataCounts,
        effective_date: NaiveDate,
        date_range: (NaiveDate, NaiveDate),
    ) -> GeneratedMasterData {
        // Generate vendors
        let vendors = self.generate_vendors(company_code, counts.vendors, effective_date);

        // Generate customers
        let customers = self.generate_customers(company_code, counts.customers, effective_date);

        // Generate materials
        let materials = self.generate_materials(company_code, counts.materials, effective_date);

        // Generate assets
        let assets = self.generate_assets(company_code, counts.assets, date_range);

        // Generate employees
        let employees = self.generate_employees(company_code, date_range);

        GeneratedMasterData {
            vendors,
            customers,
            materials,
            assets,
            employees,
            registry: self.registry.clone(),
        }
    }

    /// Generate master data for multiple company codes.
    pub fn generate_multi_company_master_data(
        &mut self,
        company_codes: &[String],
        counts: &MasterDataCounts,
        effective_date: NaiveDate,
        date_range: (NaiveDate, NaiveDate),
    ) -> Vec<GeneratedMasterData> {
        let mut results = Vec::new();

        for company_code in company_codes {
            let data = self.generate_master_data(company_code, counts, effective_date, date_range);
            results.push(data);
        }

        results
    }

    /// Generate master data with intercompany relationships.
    pub fn generate_master_data_with_ic(
        &mut self,
        company_codes: &[String],
        counts: &MasterDataCounts,
        effective_date: NaiveDate,
        date_range: (NaiveDate, NaiveDate),
    ) -> Vec<GeneratedMasterData> {
        let mut results = Vec::new();

        for company_code in company_codes {
            // Get partner company codes (all except current)
            let partners: Vec<String> = company_codes
                .iter()
                .filter(|c| *c != company_code)
                .cloned()
                .collect();

            // Generate vendors with IC
            let vendors = self.vendor_generator.generate_vendor_pool_with_ic(
                counts.vendors,
                company_code,
                &partners,
                effective_date,
            );

            // Register vendors
            for vendor in &vendors.vendors {
                self.register_vendor(vendor, company_code, effective_date);
            }

            // Generate customers with IC
            let customers = self.customer_generator.generate_customer_pool_with_ic(
                counts.customers,
                company_code,
                &partners,
                effective_date,
            );

            // Register customers
            for customer in &customers.customers {
                self.register_customer(customer, company_code, effective_date);
            }

            // Generate materials
            let materials = self.generate_materials(company_code, counts.materials, effective_date);

            // Generate assets
            let assets = self.generate_assets(company_code, counts.assets, date_range);

            // Generate employees
            let employees = self.generate_employees(company_code, date_range);

            results.push(GeneratedMasterData {
                vendors,
                customers,
                materials,
                assets,
                employees,
                registry: self.registry.clone(),
            });
        }

        results
    }

    /// Generate vendors.
    fn generate_vendors(
        &mut self,
        company_code: &str,
        count: usize,
        effective_date: NaiveDate,
    ) -> VendorPool {
        let pool = self
            .vendor_generator
            .generate_vendor_pool(count, company_code, effective_date);

        // Register each vendor in the entity registry
        for vendor in &pool.vendors {
            self.register_vendor(vendor, company_code, effective_date);
        }

        pool
    }

    /// Generate customers.
    fn generate_customers(
        &mut self,
        company_code: &str,
        count: usize,
        effective_date: NaiveDate,
    ) -> CustomerPool {
        let pool =
            self.customer_generator
                .generate_customer_pool(count, company_code, effective_date);

        // Register each customer in the entity registry
        for customer in &pool.customers {
            self.register_customer(customer, company_code, effective_date);
        }

        pool
    }

    /// Generate materials.
    fn generate_materials(
        &mut self,
        company_code: &str,
        count: usize,
        effective_date: NaiveDate,
    ) -> MaterialPool {
        let pool = self.material_generator.generate_material_pool_with_bom(
            count,
            0.25, // 25% with BOM
            company_code,
            effective_date,
        );

        // Register each material in the entity registry
        for material in &pool.materials {
            self.register_material(material, company_code, effective_date);
        }

        pool
    }

    /// Generate assets.
    fn generate_assets(
        &mut self,
        company_code: &str,
        count: usize,
        date_range: (NaiveDate, NaiveDate),
    ) -> FixedAssetPool {
        let pool = self
            .asset_generator
            .generate_diverse_pool(count, company_code, date_range);

        // Register each asset in the entity registry
        for asset in &pool.assets {
            self.register_asset(asset, company_code, asset.acquisition_date);
        }

        pool
    }

    /// Generate employees.
    fn generate_employees(
        &mut self,
        company_code: &str,
        date_range: (NaiveDate, NaiveDate),
    ) -> EmployeePool {
        let pool = self
            .employee_generator
            .generate_company_pool(company_code, date_range);

        // Register each employee in the entity registry
        for employee in &pool.employees {
            if let Some(hire_date) = employee.hire_date {
                self.register_employee(employee, company_code, hire_date);
            }
        }

        pool
    }

    /// Register a vendor in the entity registry.
    fn register_vendor(&mut self, vendor: &Vendor, company_code: &str, effective_date: NaiveDate) {
        let entity_id = EntityId::vendor(&vendor.vendor_id);
        let record =
            datasynth_core::models::EntityRecord::new(entity_id, &vendor.name, effective_date)
                .with_company_code(company_code);
        self.registry.register(record);
    }

    /// Register a customer in the entity registry.
    fn register_customer(
        &mut self,
        customer: &datasynth_core::models::Customer,
        company_code: &str,
        effective_date: NaiveDate,
    ) {
        let entity_id = EntityId::customer(&customer.customer_id);
        let record =
            datasynth_core::models::EntityRecord::new(entity_id, &customer.name, effective_date)
                .with_company_code(company_code);
        self.registry.register(record);
    }

    /// Register a material in the entity registry.
    fn register_material(
        &mut self,
        material: &Material,
        company_code: &str,
        effective_date: NaiveDate,
    ) {
        let entity_id = EntityId::material(&material.material_id);
        let record = datasynth_core::models::EntityRecord::new(
            entity_id,
            &material.description,
            effective_date,
        )
        .with_company_code(company_code);
        self.registry.register(record);
    }

    /// Register an asset in the entity registry.
    fn register_asset(
        &mut self,
        asset: &FixedAsset,
        company_code: &str,
        effective_date: NaiveDate,
    ) {
        let entity_id = EntityId::fixed_asset(&asset.asset_id);
        let record = datasynth_core::models::EntityRecord::new(
            entity_id,
            &asset.description,
            effective_date,
        )
        .with_company_code(company_code);
        self.registry.register(record);
    }

    /// Register an employee in the entity registry.
    fn register_employee(
        &mut self,
        employee: &Employee,
        company_code: &str,
        effective_date: NaiveDate,
    ) {
        let entity_id = EntityId::employee(&employee.employee_id);
        let record = datasynth_core::models::EntityRecord::new(
            entity_id,
            &employee.display_name,
            effective_date,
        )
        .with_company_code(company_code);
        self.registry.register(record);
    }

    /// Get the entity registry.
    pub fn registry(&self) -> &EntityRegistry {
        &self.registry
    }

    /// Validate that an entity exists on a given date.
    pub fn validate_entity(
        &self,
        entity_type: EntityType,
        entity_id: &str,
        transaction_date: NaiveDate,
    ) -> bool {
        let id = EntityId {
            entity_type,
            id: entity_id.to_string(),
        };
        self.registry.is_valid_on(&id, transaction_date)
    }

    /// Get active entities of a type on a given date.
    pub fn get_active_entities(&self, entity_type: EntityType, date: NaiveDate) -> Vec<EntityId> {
        self.registry
            .get_ids_by_type(entity_type)
            .into_iter()
            .filter(|id| self.registry.is_valid_on(id, date))
            .cloned()
            .collect()
    }

    /// Get a random active vendor for a company on a date.
    pub fn get_random_vendor(
        &self,
        company_code: &str,
        date: NaiveDate,
        rng: &mut impl rand::Rng,
    ) -> Option<String> {
        let vendors = self
            .registry
            .get_by_company(company_code)
            .into_iter()
            .filter(|rec| rec.entity_id.entity_type == EntityType::Vendor)
            .filter(|rec| self.registry.is_valid(&rec.entity_id, date))
            .collect::<Vec<_>>();

        if vendors.is_empty() {
            None
        } else {
            use rand::seq::IndexedRandom;
            vendors.choose(rng).map(|rec| rec.entity_id.id.clone())
        }
    }

    /// Get a random active customer for a company on a date.
    pub fn get_random_customer(
        &self,
        company_code: &str,
        date: NaiveDate,
        rng: &mut impl rand::Rng,
    ) -> Option<String> {
        let customers = self
            .registry
            .get_by_company(company_code)
            .into_iter()
            .filter(|rec| rec.entity_id.entity_type == EntityType::Customer)
            .filter(|rec| self.registry.is_valid(&rec.entity_id, date))
            .collect::<Vec<_>>();

        if customers.is_empty() {
            None
        } else {
            use rand::seq::IndexedRandom;
            customers.choose(rng).map(|rec| rec.entity_id.id.clone())
        }
    }

    /// Reset all generators.
    pub fn reset(&mut self) {
        self.vendor_generator.reset();
        self.customer_generator.reset();
        self.material_generator.reset();
        self.asset_generator.reset();
        self.employee_generator.reset();
        self.registry = EntityRegistry::new();
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = EntityRegistryManager::new(42);
        assert_eq!(manager.registry().total_count(), 0);
    }

    #[test]
    fn test_master_data_generation() {
        let mut manager = EntityRegistryManager::new(42);
        let counts = MasterDataCounts {
            vendors: 10,
            customers: 20,
            materials: 50,
            assets: 15,
            employees: None,
        };

        let data = manager.generate_master_data(
            "1000",
            &counts,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        assert_eq!(data.vendors.vendors.len(), 10);
        assert_eq!(data.customers.customers.len(), 20);
        assert_eq!(data.materials.materials.len(), 50);
        assert_eq!(data.assets.assets.len(), 15);
        assert!(!data.employees.employees.is_empty());

        // Registry should have all entities
        assert!(data.registry.total_count() > 0);
    }

    #[test]
    fn test_entity_validation() {
        let mut manager = EntityRegistryManager::new(42);
        let counts = MasterDataCounts {
            vendors: 5,
            ..Default::default()
        };

        let data = manager.generate_master_data(
            "1000",
            &counts,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        // First vendor should be valid
        let vendor_id = &data.vendors.vendors[0].vendor_id;
        assert!(manager.validate_entity(
            EntityType::Vendor,
            vendor_id,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));

        // Non-existent vendor should not be valid
        assert!(!manager.validate_entity(
            EntityType::Vendor,
            "V-NONEXISTENT",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        ));
    }

    #[test]
    fn test_multi_company_generation() {
        let mut manager = EntityRegistryManager::new(42);
        let counts = MasterDataCounts {
            vendors: 5,
            customers: 10,
            materials: 20,
            assets: 5,
            employees: None,
        };

        let results = manager.generate_multi_company_master_data(
            &["1000".to_string(), "2000".to_string()],
            &counts,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_intercompany_generation() {
        let mut manager = EntityRegistryManager::new(42);
        let counts = MasterDataCounts {
            vendors: 10,
            customers: 15,
            materials: 20,
            assets: 5,
            employees: None,
        };

        let results = manager.generate_master_data_with_ic(
            &["1000".to_string(), "2000".to_string()],
            &counts,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        // Each company should have IC vendors for the other company
        let ic_vendors: Vec<_> = results[0]
            .vendors
            .vendors
            .iter()
            .filter(|v| v.is_intercompany)
            .collect();
        assert!(!ic_vendors.is_empty());
    }

    #[test]
    fn test_get_random_vendor() {
        use datasynth_core::utils::seeded_rng;

        let mut manager = EntityRegistryManager::new(42);
        let counts = MasterDataCounts {
            vendors: 10,
            ..Default::default()
        };

        let _data = manager.generate_master_data(
            "1000",
            &counts,
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            (
                NaiveDate::from_ymd_opt(2020, 1, 1).unwrap(),
                NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            ),
        );

        let mut rng = seeded_rng(42, 0);
        let vendor = manager.get_random_vendor(
            "1000",
            NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            &mut rng,
        );

        assert!(vendor.is_some());
    }
}
