use std::path::{Path, PathBuf};

use cosmwasm_std::{Addr, Binary, Decimal};
use cw_headstash::tokenfactory::{HeadstashTokenObject, TokenStrategy};
use cw_headstash::wavs::{WavsAuthMetadata, WavsOpAuth, WavsProofOfOwnership};
use cw_infuser_scripts::suite::svg::load_svg_init_msg;
use cw_infuser_scripts::suite::whitelist::load_terp_warrior_mtree;
use cw_infuser_scripts::suite::CwSvgSuiteDeployData;
use shit_scripts::{CwShitstrapSuiteDeployData, PossibleShit, ShitInitMsg, UncheckedDenom};

/// Root of the cw-infuser repo (where `scripts/svgs/` and `data/` live).
fn cw_infuser_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../cw-infuser")
}

/// Root of the dao-contracts repo (where `artifacts/` should live).
fn dao_contracts_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../abstract/dao-contracts")
}

fn svg_init_path(collection: &str) -> String {
    cw_infuser_root()
        .join(format!("scripts/svgs/interchain/{collection}/init.json"))
        .to_string_lossy()
        .into_owned()
}

fn mtree_init_path() -> String {
    cw_infuser_root()
        .join("data/mtree-init.json")
        .to_string_lossy()
        .into_owned()
}

/// Check if DAO WASM artifacts are available.
fn dao_artifacts_available() -> bool {
    let dir = dao_contracts_root().join("artifacts");
    if dir.is_dir() {
        // Check at least dao_dao_core.wasm exists
        dir.join("dao_dao_core.wasm").exists()
            || dir.join("dao_dao_core-aarch64.wasm").exists()
    } else {
        false
    }
}

/// Preflight: verify all required data files exist before deploying.
/// Returns a list of missing paths (empty = all good).
pub fn preflight_check(full: bool) -> Vec<String> {
    let mut missing = Vec::new();

    // SVG init files
    let terp_path = svg_init_path("terp");
    if !Path::new(&terp_path).exists() {
        missing.push(terp_path);
    }

    // Merkle tree data (single mode only)
    if !full {
        let mt_path = mtree_init_path();
        if !Path::new(&mt_path).exists() {
            missing.push(mt_path);
        }
    }

    if full {
        let dao_path = svg_init_path("dao");
        if !Path::new(&dao_path).exists() {
            missing.push(dao_path);
        }
    }

    missing
}

/// Deploy configuration for the ZK headstash system.
pub struct ZkDeployData {
    /// Owner address for the manifold factory.
    pub owner: Option<String>,
    /// Instantiate message for the cw-headstash contract.
    pub headstash_init: cw_headstash::msg::InstantiateMsg,
    /// Combined VK bytes (`params || vk || cs || footer`) to register on-chain.
    /// If `None`, VK registration is skipped (must be done separately).
    pub vk_bytes: Option<Vec<u8>>,
    /// Directory containing circuit key files (params.bin, vk_combined.bin, etc.).
    pub keys_dir: Option<PathBuf>,
}

impl ZkDeployData {
    /// Create deploy data for a local test deployment.
    ///
    /// `genesis_root` is the hex-encoded merkle root of the headstash tree.
    /// `vk_combined_path` is the path to `vk_combined.bin` (optional).
    /// Create deploy data for a local test deployment.
    ///
    /// Uses `ExistingFungible` token strategy with `uterp` for simplicity.
    /// `genesis_root` is the raw merkle root bytes.
    /// `vk_combined_path` points to `vk_combined.bin` (optional).
    pub fn local_default(
        admin: Addr,
        genesis_root: &[u8],
        vk_combined_path: Option<&Path>,
    ) -> anyhow::Result<Self> {
        let vk_bytes = match vk_combined_path {
            Some(p) if p.exists() => Some(std::fs::read(p)?),
            _ => None,
        };

        Ok(Self {
            owner: Some(admin.to_string()),
            headstash_init: cw_headstash::msg::InstantiateMsg {
                genesis_root: Binary::from(genesis_root.to_vec()),
                token_strategy: TokenStrategy::ExistingFungible(HeadstashTokenObject {
                    proof: Binary::default(),
                    raw: "uterp".into(),
                }),
                wavs: WavsProofOfOwnership {
                    poos: vec![],
                    msg: WavsAuthMetadata {
                        aggregate_key: String::new(),
                        threshold: 0,
                        total_operators: 0,
                        nonce: 0,
                    },
                },
            },
            vk_bytes,
            keys_dir: None,
        })
    }
}

/// Top-level deploy configuration for all website suites.
///
/// Each field is `Option` — only suites with `Some(data)` get deployed.
pub struct TerpNetworkDeployData {
    pub admin: Addr,
    pub cw_infuser: Option<CwSvgSuiteDeployData>,
    pub terp_billboards: Option<Addr>,
    pub shitstraps: Option<CwShitstrapSuiteDeployData>,
    #[cfg(feature = "dao")]
    pub dao: Option<Addr>,
    /// ZK headstash contracts + circuits.  `None` = skip ZK deploy.
    pub zk: Option<ZkDeployData>,
}

impl TerpNetworkDeployData {
    /// Minimal local deployment: one SVG collection + one shitstrap exchange.
    pub fn local_default(sender: Addr) -> anyhow::Result<Self> {
        let svg_data = deploy_data_single(sender.clone())?;
        let shit_data = shit_deploy_data_single(sender.clone());

        #[cfg(feature = "dao")]
        let dao = if dao_artifacts_available() {
            Some(sender.clone())
        } else {
            eprintln!("WARN: DAO artifacts not found at {:?}/artifacts — skipping DaoDaoSuite", dao_contracts_root());
            eprintln!("      Build with: cd ~/abstract/dao-contracts && cargo run-script optimize");
            None
        };

        // Check for headstash circuit keys
        let keys_dir = PathBuf::from(
            std::env::var("HEADSTASH_KEYS_DIR")
                .unwrap_or_else(|_| "./circuit_keys/headstash".into()),
        );
        let vk_combined = keys_dir.join("vk_combined.bin");
        let zk = if vk_combined.exists() {
            // Use a dummy genesis root for local testing — the real root
            // comes from gen_headstash_tree output.
            let genesis_root = std::env::var("HEADSTASH_GENESIS_ROOT")
                .map(|hex| hex::decode(hex.trim_start_matches("0x")).unwrap_or_default())
                .unwrap_or_else(|_| vec![0u8; 32]);

            match ZkDeployData::local_default(sender.clone(), &genesis_root, Some(&vk_combined)) {
                Ok(data) => {
                    eprintln!("INFO: Found headstash circuit keys at {:?}", keys_dir);
                    Some(data)
                }
                Err(e) => {
                    eprintln!("WARN: Failed to load ZK deploy data: {} — skipping headstash", e);
                    None
                }
            }
        } else {
            eprintln!(
                "INFO: No headstash circuit keys at {:?} — skipping ZK deploy",
                vk_combined
            );
            eprintln!("      Generate with: cd headstash && cargo run --bin gen_headstash_keys");
            None
        };

        Ok(Self {
            admin: sender.clone(),
            cw_infuser: svg_data,
            terp_billboards: Some(sender.clone()),
            shitstraps: shit_data,
            #[cfg(feature = "dao")]
            dao,
            zk,
        })
    }

    /// Full deployment: multiple SVG collections + multiple shitstrap exchanges.
    pub fn full(sender: Addr) -> anyhow::Result<Self> {
        let svg_data = deploy_data_full(sender.clone())?;
        let shit_data = shit_deploy_data_full(sender.clone());

        #[cfg(feature = "dao")]
        let dao = if dao_artifacts_available() {
            Some(sender.clone())
        } else {
            eprintln!("WARN: DAO artifacts not found at {:?}/artifacts — skipping DaoDaoSuite", dao_contracts_root());
            eprintln!("      Build with: cd ~/abstract/dao-contracts && cargo run-script optimize");
            None
        };

        Ok(Self {
            admin: sender.clone(),
            cw_infuser: svg_data,
            terp_billboards: Some(sender.clone()),
            shitstraps: shit_data,
            #[cfg(feature = "dao")]
            dao,
            zk: None, // TODO: enable for full deploy
        })
    }
}

// ---------------------------------------------------------------------------
// cw-infuser deploy data builders
// ---------------------------------------------------------------------------

/// Single SVG collection (terp warrior) with merkle-tree whitelist.
fn deploy_data_single(sender: Addr) -> anyhow::Result<Option<CwSvgSuiteDeployData>> {
    let mut terp = load_svg_init_msg(&svg_init_path("terp"))?;
    let mut mt = load_terp_warrior_mtree(&mtree_init_path())?;
    terp.owner = Some(sender.to_string());
    mt.admins = vec![sender.to_string()];

    Ok(Some(CwSvgSuiteDeployData {
        svg: vec![(terp, Some(mt))],
        infuse: None,
        admin: Some(sender),
        infuse_coins: vec![],
        shit: None,
    }))
}

/// Multiple SVG collections (terp + dao).
fn deploy_data_full(sender: Addr) -> anyhow::Result<Option<CwSvgSuiteDeployData>> {
    let mut terp = load_svg_init_msg(&svg_init_path("terp"))?;
    let mut dao = load_svg_init_msg(&svg_init_path("dao"))?;
    terp.owner = Some(sender.to_string());
    dao.owner = Some(sender.to_string());

    Ok(Some(CwSvgSuiteDeployData {
        svg: vec![(terp, None), (dao, None)],
        infuse: None,
        admin: Some(sender),
        infuse_coins: vec![],
        shit: None,
    }))
}

// ---------------------------------------------------------------------------
// shitstrap deploy data builders
// ---------------------------------------------------------------------------

/// Single shitstrap: 20 THIOL in -> 1 TERP out.
fn shit_deploy_data_single(admin: Addr) -> Option<CwShitstrapSuiteDeployData> {
    let mut dd = CwShitstrapSuiteDeployData::default();
    dd.admin = Some(admin.clone());
    dd.shit = vec![ShitInitMsg {
        daos: Vec::new(),
        owner: Some(admin.to_string()),
        accepted: vec![PossibleShit::native_denom(
            "uthiol",
            50_000_000_000_000_000u128,
        )],
        cutoff: 500_000_000_000u128.into(),
        shitmos: UncheckedDenom::Native("uterp".into()),
        title: "terp".into(),
        description: "terp".into(),
    }];
    Some(dd)
}

/// Multiple shistraps with spot-price-based exchange rates.
fn shit_deploy_data_full(admin: Addr) -> Option<CwShitstrapSuiteDeployData> {
    let mut dd = CwShitstrapSuiteDeployData::default();
    let cut = 710_000_000_000u128;

    let mut shit = Vec::new();

    // atom @ $1.81
    shit.push(build_shit_init(
        &admin,
        &tf_denom(&admin, "atom"),
        calc_rates(Decimal::from_ratio(181u128, 100u128)),
        cut,
        "atom",
    ));

    // btc @ $66,350
    shit.push(build_shit_init(
        &admin,
        &tf_denom(&admin, "btc"),
        calc_rates(Decimal::from_ratio(66350u128, 1u128)),
        cut,
        "btc",
    ));

    // akt @ $0.29
    shit.push(build_shit_init(
        &admin,
        &tf_denom(&admin, "akt"),
        calc_rates(Decimal::from_ratio(29u128, 100u128)),
        cut,
        "akt",
    ));

    // um @ $0.007
    shit.push(build_shit_init(
        &admin,
        &tf_denom(&admin, "um"),
        calc_rates(Decimal::from_ratio(7u128, 1000u128)),
        cut,
        "um",
    ));

    // eth @ $1,956.12
    shit.push(build_shit_init(
        &admin,
        &tf_denom(&admin, "eth"),
        calc_rates(Decimal::from_ratio(195612u128, 100u128)),
        cut,
        "eth",
    ));

    dd.admin = Some(admin);
    dd.shit = shit;
    Some(dd)
}

fn calc_rates(price: Decimal) -> u128 {
    price.atomics().u128() * 100
}

fn tf_denom(creator: &Addr, subdenom: &str) -> String {
    format!("factory/{}/{}", creator, subdenom)
}

fn build_shit_init(
    admin: &Addr,
    native_denom: &str,
    shit_rate: u128,
    cutoff: u128,
    title: &str,
) -> ShitInitMsg {
    ShitInitMsg {
        daos: Vec::new(),
        owner: Some(admin.to_string()),
        accepted: vec![PossibleShit::native_denom(native_denom, shit_rate)],
        cutoff: cutoff.into(),
        shitmos: UncheckedDenom::Native("uthiol".into()),
        title: title.into(),
        description: title.into(),
    }
}
