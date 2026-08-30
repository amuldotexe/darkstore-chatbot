use std::{collections::HashSet, env};

use async_trait::async_trait;
use libsql::{Builder, Connection, Row};
use serde::{Deserialize, Serialize};

use crate::AppError;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ProductSku(String);

impl ProductSku {
    pub fn parse_product_sku(value: impl Into<String>) -> Result<Self, AppError> {
        let value = value.into();
        if value.trim().is_empty() {
            return Err(AppError::InventoryConfiguration);
        }
        Ok(Self(value))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct CatalogProduct {
    pub sku: ProductSku,
    pub category_id: String,
    pub brand: String,
    pub product_name: String,
    pub current_price_inr: i64,
    pub fixture_available: bool,
    pub fixture_sizes: Vec<String>,
    pub fixture_delivery_minutes: i64,
    pub fixture_propensity_score: i64,
    pub fixture_dress_type: String,
    pub fixture_style_tags: Vec<String>,
    pub source_product_url: String,
}

impl CatalogProduct {
    pub fn create_fixture_catalog_product(
        sku: &str,
        category_id: &str,
        score: i64,
    ) -> Result<Self, AppError> {
        Ok(Self {
            sku: ProductSku::parse_product_sku(sku)?,
            category_id: category_id.to_owned(),
            brand: "Fixture brand".to_owned(),
            product_name: format!("Fixture dress {sku}"),
            current_price_inr: 1_000,
            fixture_available: true,
            fixture_sizes: vec!["S".to_owned(), "M".to_owned()],
            fixture_delivery_minutes: 50,
            fixture_propensity_score: score,
            fixture_dress_type: "fixture_dress".to_owned(),
            fixture_style_tags: vec!["fixture".to_owned()],
            source_product_url: "https://www.slikk.club/".to_owned(),
        })
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CategoryDecision {
    Matched {
        category_id: String,
        rationale: String,
    },
    NotInInventory {
        acknowledgement: String,
    },
}

pub fn validate_model_category_decision(
    taxonomy: &[String],
    decision: CategoryDecision,
) -> Result<CategoryDecision, AppError> {
    match &decision {
        CategoryDecision::Matched {
            category_id,
            rationale,
        } if taxonomy.iter().any(|known| known == category_id) && !rationale.trim().is_empty() => {
            Ok(decision)
        }
        CategoryDecision::NotInInventory { acknowledgement }
            if !acknowledgement.trim().is_empty() =>
        {
            Ok(decision)
        }
        _ => Err(AppError::InvalidCategoryDecision),
    }
}

pub fn rank_category_product_propensity(
    records: &[CatalogProduct],
    category_id: &str,
    shown_skus: &[String],
) -> Result<Vec<CatalogProduct>, AppError> {
    let cards = rank_remaining_product_page(records, category_id, shown_skus)?;
    if cards.len() < 3 {
        return Err(AppError::CompletePageExhausted);
    }
    Ok(cards)
}

pub fn rank_remaining_product_page(
    records: &[CatalogProduct],
    category_id: &str,
    shown_skus: &[String],
) -> Result<Vec<CatalogProduct>, AppError> {
    let eligible = collect_ranked_product_candidates(records, category_id, shown_skus);
    if eligible.is_empty() {
        return Err(AppError::CompletePageExhausted);
    }

    Ok(eligible.into_iter().take(3).collect())
}

fn collect_ranked_product_candidates(
    records: &[CatalogProduct],
    category_id: &str,
    shown_skus: &[String],
) -> Vec<CatalogProduct> {
    let shown: HashSet<&str> = shown_skus.iter().map(String::as_str).collect();
    let mut eligible: Vec<_> = records
        .iter()
        .filter(|product| {
            product.category_id == category_id
                && product.fixture_available
                && !shown.contains(product.sku.as_str())
        })
        .cloned()
        .collect();

    eligible.sort_by(|left, right| {
        right
            .fixture_propensity_score
            .cmp(&left.fixture_propensity_score)
            .then_with(|| left.sku.cmp(&right.sku))
    });

    eligible
}

pub fn determine_unseen_product_availability(
    records: &[CatalogProduct],
    category_id: &str,
    shown_skus: &[String],
) -> bool {
    rank_remaining_product_page(records, category_id, shown_skus).is_ok()
}

#[async_trait]
pub trait CatalogRepository: Send + Sync {
    async fn load_runtime_inventory_taxonomy(&self) -> Result<Vec<String>, AppError>;
    async fn load_catalog_product_records(&self) -> Result<Vec<CatalogProduct>, AppError>;
}

/// Read-only v001 catalogue compiled into the desktop binary.
///
/// The release path deliberately has no dependency on user-shell configuration or a remote
/// database. The source projection mirrors the checked-in Turso-compatible seed data.
#[derive(Debug)]
pub struct EmbeddedCatalogRepository {
    records: Result<Vec<CatalogProduct>, AppError>,
}

impl EmbeddedCatalogRepository {
    pub fn create_embedded_catalog_repository() -> Self {
        Self {
            records: Self::load_embedded_catalog_records(),
        }
    }

    fn load_embedded_catalog_records() -> Result<Vec<CatalogProduct>, AppError> {
        let records: Vec<CatalogProduct> =
            serde_json::from_str(include_str!("../../data/darkstore-dresses-v001.json"))
                .map_err(|_| AppError::InventoryConfiguration)?;
        let mut taxonomy: Vec<String> = records
            .iter()
            .map(|product| product.category_id.clone())
            .collect();
        taxonomy.sort();
        taxonomy.dedup();

        if records.len() != 8
            || taxonomy != ["dresses"]
            || records.iter().any(|product| {
                product.category_id.trim().is_empty()
                    || product.product_name.trim().is_empty()
                    || !(2..=3).contains(&product.fixture_sizes.len())
            })
        {
            return Err(AppError::InventoryConfiguration);
        }

        Ok(records)
    }
}

#[async_trait]
impl CatalogRepository for EmbeddedCatalogRepository {
    async fn load_runtime_inventory_taxonomy(&self) -> Result<Vec<String>, AppError> {
        let mut taxonomy: Vec<String> = self
            .records
            .as_ref()
            .map_err(|_| AppError::InventoryConfiguration)?
            .iter()
            .map(|product| product.category_id.clone())
            .collect();
        taxonomy.sort();
        taxonomy.dedup();
        Ok(taxonomy)
    }

    async fn load_catalog_product_records(&self) -> Result<Vec<CatalogProduct>, AppError> {
        self.records
            .as_ref()
            .map(Clone::clone)
            .map_err(|_| AppError::InventoryConfiguration)
    }
}

/// Non-serializable, backend-only Turso configuration.
///
/// Its fields stay private so a connection token cannot become a frontend DTO by accident.
pub struct TursoConnectionConfiguration {
    database_url: String,
    auth_token: String,
}

pub fn parse_turso_connection_configuration(
    database_url: Option<&str>,
    auth_token: Option<&str>,
) -> Result<TursoConnectionConfiguration, AppError> {
    let database_url = database_url
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::InventoryUnavailable)?;
    let auth_token = auth_token
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .ok_or(AppError::InventoryUnavailable)?;

    Ok(TursoConnectionConfiguration {
        database_url: database_url.to_owned(),
        auth_token: auth_token.to_owned(),
    })
}

/// L3 remote data adapter. It reads environment configuration only when a command needs data,
/// leaving desktop startup offline and key-required.
#[derive(Default)]
pub struct TursoCatalogRepository;

impl TursoCatalogRepository {
    pub fn create_turso_catalog_repository() -> Self {
        Self
    }
}

#[async_trait]
impl CatalogRepository for TursoCatalogRepository {
    async fn load_runtime_inventory_taxonomy(&self) -> Result<Vec<String>, AppError> {
        let connection = open_remote_turso_connection().await?;
        let mut rows = connection
            .query(
                "SELECT DISTINCT category_id FROM inventory_products ORDER BY category_id ASC",
                (),
            )
            .await
            .map_err(|_| AppError::InventoryUnavailable)?;
        let mut taxonomy = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|_| AppError::InventoryUnavailable)?
        {
            let category_id = row
                .get::<String>(0)
                .map_err(|_| AppError::InventoryUnavailable)?;
            if category_id.trim().is_empty() {
                return Err(AppError::InventoryUnavailable);
            }
            taxonomy.push(category_id);
        }

        if taxonomy.is_empty() {
            return Err(AppError::InventoryUnavailable);
        }
        Ok(taxonomy)
    }

    async fn load_catalog_product_records(&self) -> Result<Vec<CatalogProduct>, AppError> {
        let connection = open_remote_turso_connection().await?;
        let mut rows = connection
            .query(
                "SELECT sku, category_id, brand, product_name, current_price_inr, \
                 fixture_available, fixture_sizes_json, fixture_delivery_minutes, \
                 fixture_propensity_score, fixture_dress_type, fixture_style_tags_json, \
                 source_product_url FROM inventory_products ORDER BY sku ASC",
                (),
            )
            .await
            .map_err(|_| AppError::InventoryUnavailable)?;
        let mut records = Vec::new();

        while let Some(row) = rows
            .next()
            .await
            .map_err(|_| AppError::InventoryUnavailable)?
        {
            records.push(parse_turso_catalog_row(&row)?);
        }

        Ok(records)
    }
}

async fn open_remote_turso_connection() -> Result<Connection, AppError> {
    let configuration = read_runtime_turso_configuration()?;
    let database = Builder::new_remote(configuration.database_url, configuration.auth_token)
        .build()
        .await
        .map_err(|_| AppError::InventoryUnavailable)?;
    database
        .connect()
        .map_err(|_| AppError::InventoryUnavailable)
}

fn read_runtime_turso_configuration() -> Result<TursoConnectionConfiguration, AppError> {
    let database_url = env::var("TURSO_DATABASE_URL").ok();
    let auth_token = env::var("TURSO_AUTH_TOKEN").ok();
    parse_turso_connection_configuration(database_url.as_deref(), auth_token.as_deref())
}

fn parse_turso_catalog_row(row: &Row) -> Result<CatalogProduct, AppError> {
    let sku = row
        .get::<String>(0)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let category_id = row
        .get::<String>(1)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let brand = row
        .get::<String>(2)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let product_name = row
        .get::<String>(3)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let current_price_inr = row
        .get::<i64>(4)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let fixture_available = match row
        .get::<i64>(5)
        .map_err(|_| AppError::InventoryUnavailable)?
    {
        0 => false,
        1 => true,
        _ => return Err(AppError::InventoryUnavailable),
    };
    let fixture_sizes_json = row
        .get::<String>(6)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let fixture_delivery_minutes = row
        .get::<i64>(7)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let fixture_propensity_score = row
        .get::<i64>(8)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let fixture_dress_type = row
        .get::<String>(9)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let fixture_style_tags_json = row
        .get::<String>(10)
        .map_err(|_| AppError::InventoryUnavailable)?;
    let source_product_url = row
        .get::<String>(11)
        .map_err(|_| AppError::InventoryUnavailable)?;

    let fixture_sizes = parse_json_string_list(&fixture_sizes_json)?;
    let fixture_style_tags = parse_json_string_list(&fixture_style_tags_json)?;
    if category_id.trim().is_empty()
        || brand.trim().is_empty()
        || product_name.trim().is_empty()
        || current_price_inr < 0
        || fixture_sizes.len() < 2
        || fixture_sizes.len() > 3
        || fixture_delivery_minutes <= 0
        || source_product_url.trim().is_empty()
    {
        return Err(AppError::InventoryUnavailable);
    }

    Ok(CatalogProduct {
        sku: ProductSku::parse_product_sku(sku).map_err(|_| AppError::InventoryUnavailable)?,
        category_id,
        brand,
        product_name,
        current_price_inr,
        fixture_available,
        fixture_sizes,
        fixture_delivery_minutes,
        fixture_propensity_score,
        fixture_dress_type,
        fixture_style_tags,
        source_product_url,
    })
}

fn parse_json_string_list(encoded_value: &str) -> Result<Vec<String>, AppError> {
    let values: Vec<String> =
        serde_json::from_str(encoded_value).map_err(|_| AppError::InventoryUnavailable)?;
    if values.iter().any(|value| value.trim().is_empty()) {
        return Err(AppError::InventoryUnavailable);
    }
    Ok(values)
}
