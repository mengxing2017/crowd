mod application_boundaries;
mod catalog_domain;
mod community_domain;
mod community_postgres;
mod domain_boundaries;
mod domain_compatibility;
mod finance_domain;
mod finance_postgres;
mod http_contract;
mod isolated_identity;
mod layer_dependencies;
mod module_boundaries;
mod public_surface;
mod receipt_management;
mod tasks_domain;

#[path = "support/tasks.rs"]
mod task_support;

mod operations_domain;
mod operations_postgres;
