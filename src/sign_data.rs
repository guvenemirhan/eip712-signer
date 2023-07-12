use ethers::{
    prelude::abigen,
    providers::{Http, Provider},
};
use ethers_core::types::{
    transaction::eip712::{EIP712Domain, Eip712DomainType, TypedData, Types},
    Address, H160, U256,
};
use ethers_signers::{LocalWallet, Signer};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::{
    env,
    error::Error,
    fmt,
    ops::{Add, Mul},
    str::FromStr,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

/// Represents a presale event in a decentralized fundraising campaign.
///
/// # Fields
///
/// * `currency`: The token that will be used for the presale.
/// * `presale_rate`: The rate at which the tokens will be sold during the presale.
/// * `softcap`: The minimum amount that needs to be raised for the presale to be considered successful.
/// * `hardcap`: The maximum amount that can be raised during the presale.
/// * `min_buy`: The minimum amount that a participant can buy during the presale.
/// * `max_buy`: The maximum amount that a participant can buy during the presale.
/// * `liquidity_rate`: The percentage of the funds raised that will be allocated to the liquidity pool.
/// * `listing_rate`: The rate at which the tokens will be listed on the exchange after the presale.
/// * `start_time`: The start time of the presale.
/// * `end_time`: The end time of the presale.
/// * `lock_end_time`: The time until which the raised funds will be locked.
/// * `is_vesting`: A flag indicating whether the tokens will be vested or not.
/// * `is_lock`: A flag indicating whether the raised funds will be locked or not.
/// * `refund`: A flag indicating whether the participants can request a refund if the softcap is not reached.
/// * `auto_listing`: A flag indicating whether the token will be automatically listed on the exchange after the presale.
///
/// Each field is public and can be accessed directly.
#[derive(Serialize, Deserialize, Debug)]
pub struct Presale {
    pub currency: String,
    #[serde(rename = "presaleRate")]
    pub presale_rate: u64,
    pub softcap: u64,
    pub hardcap: u64,
    #[serde(rename = "minBuy")]
    pub min_buy: u64,
    #[serde(rename = "maxBuy")]
    pub max_buy: u64,
    #[serde(rename = "liquidityRate")]
    pub liquidity_rate: u64,
    #[serde(rename = "listingRate")]
    pub listing_rate: u64,
    #[serde(rename = "startTime")]
    pub start_time: u64,
    #[serde(rename = "endTime")]
    pub end_time: u64,
    #[serde(rename = "lockEndTime")]
    pub lock_end_time: u64,
    #[serde(rename = "isVesting")]
    pub is_vesting: bool,
    #[serde(rename = "isLock")]
    pub is_lock: bool,
    pub refund: bool,
    #[serde(rename = "autoListing")]
    pub auto_listing: bool,
}


#[derive(Debug)]
pub enum ParamsErrors {
    MinBuyError,
    MaxBuyError,
    HardcapError,
    SoftcapError,
    LiqRateError,
    ListingRateError,
    PresaleRateError,
    StartTimeError,
    EndTimeError,
}

/// Error display implementation for ParamsErrors.
/// This trait implementation is used when ParamsErrors is to be printed
/// in a user-friendly manner.
impl fmt::Display for ParamsErrors {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

/// This trait implementation marks ParamsErrors as an Error type.
/// It allows ParamsErrors to be used where std::error::Error is expected.
impl Error for ParamsErrors {}

/// Trait to convert a bool into a Result.
/// Provides a method `ok_or` that transforms true to Ok(()) and false to Err(error).
trait ToOption {
    /// If the value is true, returns `Ok(())`.
    /// If the value is false, returns `Err(error)`.
    fn ok_or(self, error: ParamsErrors) -> Result<(), ParamsErrors>;
}

/// Implement the ToOption trait for the bool type.
/// This implementation allows any bool to be transformed into a Result<(), ParamsErrors>.
impl ToOption for bool {
    fn ok_or(self, error: ParamsErrors) -> Result<(), ParamsErrors> {
        if self {
            Ok(())
        } else {
            Err(error)
        }
    }
}

/// Checks various constraints on a `Presale` struct.
///
/// This function verifies a series of conditions, ensuring that the `Presale` data
/// is valid and well-formed. The following checks are performed:
/// - `min_buy` is greater than 0
/// - `max_buy` is greater than 0 and more than `min_buy`
/// - `max_buy` is less than or equal to `hardcap`
/// - `hardcap` is greater than 0 and more than `softcap`
/// - `softcap` is at least half of `hardcap` and is greater than 0
/// - `liquidity_rate` is between 50 and 100
/// - `listing_rate` is between 0 and 100_000_000_000
/// - `presale_rate` is between 0 and 100_000_000_000
/// - `start_time` is in the future and less than `end_time`
///
/// If all the conditions are met, it will return `Ok(())`. If a condition is not met,
/// it will return `Err` with the corresponding `ParamsErrors` variant.
///
/// # Arguments
///
/// * `presale` - The `Presale` struct that will be checked.
///
/// # Returns
///
/// `Result<(), ParamsErrors>` - If all checks pass, `Ok(())` is returned.
/// Otherwise, `Err(ParamsErrors)` is returned, where `ParamsErrors` is the specific error variant.
fn check_params(presale: &Presale) -> Result<(), ParamsErrors> {
    // Retrieves the current time's timestamp.
    let start = SystemTime::now();
    let now_timestamp = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    // A series of checks is performed with the following `and_then` calls.
    // If a check is successful, `Ok(())` is returned.
    Ok(())
        .and_then(|_| (presale.min_buy > 0).ok_or(ParamsErrors::MinBuyError))
        .and_then(|_| {
            (presale.max_buy > 0 && presale.min_buy < presale.max_buy)
                .ok_or(ParamsErrors::MaxBuyError)
        })
        .and_then(|_| (presale.max_buy <= presale.hardcap).ok_or(ParamsErrors::MaxBuyError))
        .and_then(|_| {
            (presale.hardcap > 0 && presale.hardcap > presale.softcap)
                .ok_or(ParamsErrors::HardcapError)
        })
        .and_then(|_| (presale.softcap >= presale.hardcap / 2).ok_or(ParamsErrors::HardcapError))
        .and_then(|_| (presale.softcap > 0).ok_or(ParamsErrors::SoftcapError))
        .and_then(|_| {
            (presale.liquidity_rate > 50 && presale.liquidity_rate <= 100)
                .ok_or(ParamsErrors::LiqRateError)
        })
        .and_then(|_| {
            (presale.listing_rate > 0 && presale.listing_rate <= 100_000_000_000)
                .ok_or(ParamsErrors::ListingRateError)
        })
        .and_then(|_| {
            (presale.presale_rate > 0 && presale.presale_rate <= 100_000_000_000)
                .ok_or(ParamsErrors::PresaleRateError)
        })
        .and_then(|_| {
            (presale.start_time >= now_timestamp.as_secs() && presale.start_time < presale.end_time)
                .ok_or(ParamsErrors::StartTimeError)
        })
}

/// Calculates the total amount of tokens required for the presale and liquidity provision.
///
/// This function calculates the total amount of tokens that are needed for the presale
/// and for providing liquidity, based on the hardcap, the presale rate, and the listing rate.
///
/// # Arguments
///
/// * `hardcap` - The maximum amount that can be raised during the presale.
/// * `presale_rate` - The rate at which the tokens will be sold during the presale.
/// * `listing_rate` - The rate at which the tokens will be listed on the exchange after the presale.
///
/// # Returns
///
/// `u64` - The total amount of tokens required for the presale and liquidity provision.
fn calculate_amount(hardcap: &u64, presale_rate: &u64, listing_rate: u64) -> u64 {
    let presale_amount = hardcap.mul(presale_rate);
    let liquidity_amount = hardcap.mul(listing_rate);
    presale_amount.add(liquidity_amount)
}

/// Fetches token balance and allowance information from a EVM.
///
/// This function queries the balance and the allowance of a certain address for a
/// specific token using the Ethereum network. It does this using the Ethereum RPC URL
/// and a smart contract with the ABI of the ERC20 standard.
///
/// The function will return an error if the balance or allowance of the address for
/// the token is less than the required amount.
///
/// # Arguments
///
/// * `address` - The address of the token contract.
/// * `owner` - The address of the token holder.
/// * `amount` - The minimum required balance and allowance.
///
/// # Returns
///
/// `Result<(), Box<dyn Error>>` - Returns `Ok(())` if the balance and allowance are
/// greater than or equal to the required amount. Otherwise, it returns an error.
async fn get_token_info(
    address: Address,
    owner: Address,
    amount: u64,
) -> Result<(), Box<dyn Error>> {
    let rpc_url = env::var("RPC_URL")?;
    let proxy_man = env::var("PROXY_MANAGER")?;
    let pool_man = H160::from_str(&*proxy_man).expect("Invalid address");
    let pool_manager = Address::from(pool_man);
    let provider = Provider::<Http>::try_from(rpc_url)?;
    let client = Arc::new(provider);
    abigen!(IERC20, "./src/abi/ERC20.json");
    let contract = IERC20::new(address, client);
    let balance = contract.balance_of(owner.clone()).call().await?;
    let allowance = contract.allowance(owner, pool_manager).call().await?;
    let success = balance >= U256::from(amount) && allowance >= U256::from(amount);
    if success {
        Ok(())
    } else {
        Err(Box::new(std::io::Error::new(
            std::io::ErrorKind::Other,
            "insufficient balance",
        )))
    }
}

/// Signs a given presale request using a private key.
///
/// This function first checks if the parameters of the presale request are valid. If they are not,
/// it will return an error.
///
/// After the check, it calculates the total amount of tokens that will be sold during the presale
/// and in the liquidity pool.
///
/// It then fetches the token balance and allowance information of the owner. If the balance or
/// allowance is not enough, it will return an error.
///
/// Finally, it prepares a typed data according to EIP-712 standard, and signs it with the private
/// key obtained from the environment variables.
///
/// # Arguments
///
/// * `presale` - The presale request to be signed.
/// * `owner` - The owner of the tokens to be sold.
///
/// # Return Value
///
/// `Result<String, Box<dyn std::error::Error>>` - If the presale request is successfully signed,
/// it returns the signature as a hexadecimal string. Otherwise, it returns an error.
pub async fn sign(presale: Presale, owner: Address) -> Result<String, Box<dyn std::error::Error>> {
    match check_params(&presale) {
        Ok(_) => {}
        Err(e) => return Err(Box::new(e)),
    }

    let amount = calculate_amount(
        &presale.hardcap,
        &presale.presale_rate,
        presale.listing_rate,
    );
    let currency_h160 = H160::from_str(&*presale.currency).expect("Invalid address");

    match get_token_info(Address::from(currency_h160), owner, amount).await {
        Ok(_) => {}
        Err(_) => return Err(Box::new(ParamsErrors::EndTimeError)),
    }
    let domain = EIP712Domain {
        name: Option::from(String::from("EIP712-Derive")),
        version: Option::from(String::from("1")),
        chain_id: Option::from(U256::from(1)),
        verifying_contract: Option::from(
            "/*CONTRACT ADDRESS IS HERE*/"
                .parse::<Address>()
                .expect("Invalid contract"),
        ),
        salt: None,
    };


    let domain_vec = vec![
        Eip712DomainType {
            name: "name".parse().unwrap(),
            r#type: "string".to_string(),
        },
        Eip712DomainType {
            name: "version".parse().unwrap(),
            r#type: "string".to_string(),
        },
        Eip712DomainType {
            name: "chain_id".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "verifying_contract".parse().unwrap(),
            r#type: "address".to_string(),
        },
    ];

    let permit = vec![
        Eip712DomainType {
            name: "currency".parse().unwrap(),
            r#type: "address".to_string(),
        },
        Eip712DomainType {
            name: "presaleRate".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "softcap".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "hardcap".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "minBuy".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "maxBuy".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "liquidityRate".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "listingRate".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "startTime".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "endTime".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "lockEndTime".parse().unwrap(),
            r#type: "uint256".to_string(),
        },
        Eip712DomainType {
            name: "isVesting".parse().unwrap(),
            r#type: "bool".to_string(),
        },
        Eip712DomainType {
            name: "isLock".parse().unwrap(),
            r#type: "bool".to_string(),
        },
        Eip712DomainType {
            name: "refund".parse().unwrap(),
            r#type: "bool".to_string(),
        },
        Eip712DomainType {
            name: "autoListing".parse().unwrap(),
            r#type: "bool".to_string(),
        },
    ];

    let mut types: Types = BTreeMap::new();
    types.insert("EIP712Domain".parse().unwrap(), domain_vec);
    types.insert("Permit".parse().unwrap(), permit);

    let value;
    match serde_json::to_value(presale) {
        Ok(val) => { value = val; }
        Err(e) => { return Err(Box::new(e)); }
    }
    let json_map;
    match value.as_object() {
        Some(map) => { json_map = map; }
        None => { return Err(Box::new(std::io::Error::new(std::io::ErrorKind::Other, "Not an object"))) }
    }
    let presale_data: BTreeMap<String, serde_json::Value> =
        BTreeMap::from_iter(json_map.clone().into_iter());

    let typed_data: TypedData = TypedData {
        domain,
        types,
        primary_type: "Permit".parse().unwrap(),
        message: presale_data,
    };

    match env::var("PRIVATE_KEY").unwrap().parse::<LocalWallet>() {
        Ok(wallet) => match wallet.sign_typed_data(&typed_data).await {
            Ok(signature) => Ok(format!("0x{}", signature.to_string())),
            Err(e) => Err(Box::try_from(e).unwrap()),
        },
        Err(e) => Err(Box::try_from(e).unwrap()),
    }
}
