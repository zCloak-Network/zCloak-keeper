pub trait VerifyChain {
    const CHAIN_CATEGORY:  ChainCategory;
}

pub enum ChainCategory {
    Substrate,
}