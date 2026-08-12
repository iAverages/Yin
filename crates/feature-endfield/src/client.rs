use reqwest::Client;
use serde::de::DeserializeOwned;

use crate::model::{
    CodexType, EquipmentSet, Manifest, OperatorIndexEntry, OperatorPayload, Weapon,
    WeaponIndexEntry,
};

pub const BASE_URL: &str = "https://akendfield.tools";
const MANIFEST_PATH: &str = "/data/codex-chunks/manifest.json";

pub struct CodexClient {
    http: Client,
}

impl CodexClient {
    pub fn new() -> Self {
        Self {
            http: Client::new(),
        }
    }

    pub async fn manifest(&self) -> reqwest::Result<Manifest> {
        self.get(MANIFEST_PATH).await
    }

    pub async fn operators(&self, manifest: &Manifest) -> reqwest::Result<Vec<OperatorIndexEntry>> {
        self.get(&manifest.operators.index).await
    }

    pub async fn weapons(&self, manifest: &Manifest) -> reqwest::Result<Vec<WeaponIndexEntry>> {
        self.get(&manifest.weapons.index).await
    }

    pub async fn equipment(&self, manifest: &Manifest) -> reqwest::Result<Vec<EquipmentSet>> {
        self.get(&manifest.equipment).await
    }

    pub async fn operator(
        &self,
        manifest: &Manifest,
        id: &str,
    ) -> reqwest::Result<Option<OperatorPayload>> {
        let Some(path) = manifest.operators.files.get(id) else {
            return Ok(None);
        };
        self.get(path).await.map(Some)
    }

    pub async fn weapon(&self, manifest: &Manifest, id: &str) -> reqwest::Result<Option<Weapon>> {
        let Some(path) = manifest.weapons.files.get(id) else {
            return Ok(None);
        };
        self.get(path).await.map(Some)
    }

    async fn get<T: DeserializeOwned>(&self, path: &str) -> reqwest::Result<T> {
        self.http
            .get(absolute_url(path))
            .send()
            .await?
            .error_for_status()?
            .json()
            .await
    }
}

pub fn item_url(kind: CodexType, slug: &str) -> String {
    format!("{BASE_URL}/codex/{}/{slug}", kind.path())
}

pub fn asset_url(path: &str) -> String {
    absolute_url(path)
}

fn absolute_url(path: &str) -> String {
    if path.starts_with("http://") || path.starts_with("https://") {
        path.to_owned()
    } else {
        format!("{BASE_URL}{path}")
    }
}
