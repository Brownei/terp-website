use std::collections::HashMap;

use cw_headstash::interface::HeadstashContract;
use cw_headstash_manifold::interface::CwHeadstashManifold;
use cw_infuser_scripts::suite::CwSvgSuite;
use cw_orch::prelude::*;
// #[cfg(feature = "dao")]
// use dao_testing::DaoDaoSuite;
use shit_scripts::CwShitstrapSuite;
use terp_account_scripts::TerpAccountSuite;

use crate::deploy_data::TerpNetworkDeployData;

/// ZK headstash deployment suite: cw-headstash contract + manifold factory.
///
/// Wraps the cw-orch interfaces for both contracts and provides
/// a `deploy_on` constructor matching the pattern used by other suites.
pub struct ZkHeadstashSuite<Chain: CwEnv> {
    pub headstash: HeadstashContract<Chain>,
    pub manifold: CwHeadstashManifold<Chain>,
}

impl<Chain: CwEnv> ZkHeadstashSuite<Chain> {
    /// Deploy the headstash manifold factory + headstash contract,
    /// and optionally register the circuit verifying key.
    pub fn deploy_on(
        chain: Chain,
        data: crate::deploy_data::ZkDeployData,
    ) -> Result<Self, CwOrchError> {
        // 1. Upload + instantiate the manifold factory
        let manifold = CwHeadstashManifold::new(chain.clone());
        manifold.upload()?;

        // 2. Upload the headstash contract code
        let headstash = HeadstashContract::new(chain.clone());
        headstash.upload()?;
        let headstash_code_id = headstash.code_id()?;

        // 3. Instantiate manifold with headstash code_id
        manifold.instantiate(
            &cw_headstash_manifold::msg::InstantiateMsg {
                owner: data.owner.clone(),
                headstash_code_id,
            },
            None,
            &[],
        )?;

        // 4. Instantiate the headstash contract directly
        headstash.instantiate(&data.headstash_init, None, &[])?;

        // 5. Register the VK if provided
        if let Some(vk_bytes) = data.vk_bytes {
            headstash.execute(
                &cw_headstash::msg::ExecuteMsg::LoadVk {
                    vk: cosmwasm_std::Binary::from(vk_bytes),
                },
                &[],
            )?;
        }

        Ok(Self {
            headstash,
            manifold,
        })
    }
}

/// Unified deployment suite composing all website contract suites.
pub struct TerpNetworkSuite<Chain: CwEnv> {
    pub chain: Chain,
    pub infuser: Option<CwSvgSuite<Chain>>,
    pub billboards: Option<TerpAccountSuite<Chain>>,
    // #[cfg(feature = "dao")]
    // pub dao: Option<DaoDaoSuite<Chain>>,
    pub headstash: Option<ZkHeadstashSuite<Chain>>,
}

impl<Chain: CwEnv> TerpNetworkSuite<Chain> {
    /// Deploy suites conditionally based on which deploy data fields are `Some`.
    pub fn deploy_on(chain: Chain, data: TerpNetworkDeployData) -> Result<Self, CwOrchError> {
        // 1. CwSvgSuite (cw-infuser + SVG collections + shitstraps)
        let infuser = if let Some(svg_data) = data.cw_infuser {
            let mut suite = CwSvgSuite::deploy_on(chain.clone(), Some(svg_data))?;
            if let Some(shit_data) = data.shitstraps {
                suite.shit = CwShitstrapSuite::deploy_on(chain.clone(), Some(shit_data))?;
            }
            Some(suite)
        } else {
            None
        };

        // 2. TerpAccountSuite (account NFTs + manifold minter)
        let billboards = if let Some(admin) = data.terp_billboards {
            Some(TerpAccountSuite::deploy_on(chain.clone(), admin)?)
        } else {
            None
        };

        // // 3. DaoDaoSuite (DAO contracts including calendar)
        // #[cfg(feature = "dao")]
        // let dao = if let Some(admin) = data.dao {
        //     Some(DaoDaoSuite::deploy_on(chain.clone(), admin)?)
        // } else {
        //     None
        // };

        // 4. ZkHeadstashSuite (cw-headstash + cw-headstash-manifold + circuits)
        let headstash = if let Some(zk_data) = data.zk {
            match ZkHeadstashSuite::deploy_on(chain.clone(), zk_data) {
                Ok(suite) => Some(suite),
                Err(e) => {
                    tracing::warn!("ZK headstash deploy failed (non-fatal): {}", e);
                    None
                }
            }
        } else {
            None
        };

        Ok(Self {
            chain,
            infuser,
            billboards,
            // #[cfg(feature = "dao")]
            // dao,
            headstash,
        })
    }

    /// Collect deployed contract addresses as a map (config key -> address).
    pub fn collect_addresses(&self) -> HashMap<String, String> {
        let mut addrs = HashMap::new();

        if let Some(ref suite) = self.infuser {
            if let Ok(addr) = suite.minter.addr_str() { addrs.insert("cwSvgMinter".into(), addr); }
            if let Ok(addr) = suite.cwsvg.addr_str() { addrs.insert("cw721Svg".into(), addr); }
            if let Ok(addr) = suite.infuser.addr_str() { addrs.insert("cwInfusionMinter".into(), addr); }
            if let Ok(addr) = suite.shit.factory.addr_str() { addrs.insert("shitstrapFactory".into(), addr); }
        }

        if let Some(ref suite) = self.billboards {
            if let Ok(addr) = suite.manifold.addr_str() { addrs.insert("accountMinter".into(), addr); }
            if let Ok(addr) = suite.nft.addr_str() { addrs.insert("terp721Account".into(), addr); }
        }

        // #[cfg(feature = "dao")]
        // if let Some(ref suite) = self.dao {
        //     if let Ok(addr) = suite.dao_core.addr_str() { addrs.insert("daoCore".into(), addr); }
        //     if let Ok(addr) = suite.external.calendar.addr_str() { addrs.insert("daoCalendar".into(), addr); }
        // }

        if let Some(ref suite) = self.headstash {
            if let Ok(addr) = suite.headstash.addr_str() { addrs.insert("cwHeadstash".into(), addr); }
            if let Ok(addr) = suite.manifold.addr_str() { addrs.insert("cwHeadstashManifold".into(), addr); }
        }

        addrs
    }

    /// Print deployed contract addresses in `CONTRACT_ADDR:name=addr` format.
    pub fn print_addresses(&self) {
        for (key, addr) in self.collect_addresses() {
            println!("CONTRACT_ADDR:{}={}", key, addr);
        }
    }
}
