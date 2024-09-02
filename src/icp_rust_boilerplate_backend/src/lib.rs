#[macro_use]
extern crate serde;
use candid::{Decode, Encode};
use ic_cdk::api::time;
use ic_stable_structures::memory_manager::{MemoryId, MemoryManager, VirtualMemory};
use ic_stable_structures::{BoundedStorable, Cell, DefaultMemoryImpl, StableBTreeMap, Storable};
use std::{borrow::Cow, cell::RefCell};

type Memory = VirtualMemory<DefaultMemoryImpl>;
type IdCell = Cell<u64, Memory>;

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Debt {
    id: u64,
    debtor: String,
    creditor: String,
    amount: u64,
    created_at: u64,
}

impl Storable for Debt {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Debt {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct Escrow {
    debt_id: u64,
    amount: u64,
    created_at: u64,
}

impl Storable for Escrow {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for Escrow {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct CropInsurance {
    id: u64,
    farmer: String,
    crop_type: String,
    coverage_amount: u64,
    coverage_start_date: u64,
    coverage_end_date: u64,
}

impl Storable for CropInsurance {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for CropInsurance {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

#[derive(candid::CandidType, Serialize, Deserialize, Default, Clone)]
struct InsuranceClaim {
    insurance_id: u64,
    claim_amount: u64,
    claim_date: u64,
}

impl Storable for InsuranceClaim {
    fn to_bytes(&self) -> std::borrow::Cow<[u8]> {
        Cow::Owned(Encode!(self).unwrap())
    }

    fn from_bytes(bytes: std::borrow::Cow<[u8]>) -> Self {
        Decode!(bytes.as_ref(), Self).unwrap()
    }
}

impl BoundedStorable for InsuranceClaim {
    const MAX_SIZE: u32 = 1024;
    const IS_FIXED_SIZE: bool = false;
}

thread_local! {
    static MEMORY_MANAGER: RefCell<MemoryManager<DefaultMemoryImpl>> = RefCell::new(
        MemoryManager::init(DefaultMemoryImpl::default())
    );

    static ID_COUNTER: RefCell<IdCell> = RefCell::new(
        IdCell::init(MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(0))), 0)
            .expect("Cannot create a counter")
    );

    static DEBT_STORAGE: RefCell<StableBTreeMap<u64, Debt, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(1)))
    ));

    static ESCROW_STORAGE: RefCell<StableBTreeMap<u64, Escrow, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(2)))
    ));

    static CROP_INSURANCE_STORAGE: RefCell<StableBTreeMap<u64, CropInsurance, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(3)))
    ));

    static INSURANCE_CLAIM_STORAGE: RefCell<StableBTreeMap<u64, InsuranceClaim, Memory>> =
        RefCell::new(StableBTreeMap::init(
            MEMORY_MANAGER.with(|m| m.borrow().get(MemoryId::new(4)))
    ));
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct DebtPayload {
    debtor: String,
    creditor: String,
    amount: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct EscrowPayload {
    debt_id: u64,
    amount: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct CropInsurancePayload {
    farmer: String,
    crop_type: String,
    coverage_amount: u64,
    coverage_start_date: u64,
    coverage_end_date: u64,
}

#[derive(candid::CandidType, Serialize, Deserialize, Default)]
struct InsuranceClaimPayload {
    insurance_id: u64,
    claim_amount: u64,
}

#[derive(candid::CandidType, Deserialize, Serialize)]
enum Error {
    NotFound { msg: String },
    InvalidInput { msg: String },
    InternalError { msg: String },
}

/// Validates that a string field is not empty, returning an `Error::InvalidInput` if it is.
fn validate_non_empty(field: &str, field_name: &str) -> Result<(), Error> {
    if field.is_empty() {
        Err(Error::InvalidInput {
            msg: format!("Field '{}' cannot be empty", field_name),
        })
    } else {
        Ok(())
    }
}

/// Generates a unique ID by incrementing the ID counter.
fn generate_unique_id() -> Result<u64, Error> {
    ID_COUNTER
        .with(|counter| {
            let current_value = *counter.borrow().get();
            counter
                .borrow_mut()
                .set(current_value + 1)
                .map(|_| current_value + 1)
        })
        .map_err(|_| Error::InternalError {
            msg: "Failed to generate unique ID".to_string(),
        })
}

#[ic_cdk::query]
fn get_debt(id: u64) -> Result<Debt, Error> {
    _get_debt(&id).ok_or(Error::NotFound {
        msg: format!("A debt with id={} not found", id),
    })
}

#[ic_cdk::query]
fn get_escrow(debt_id: u64) -> Result<Escrow, Error> {
    _get_escrow(&debt_id).ok_or(Error::NotFound {
        msg: format!("Escrow for debt_id={} not found", debt_id),
    })
}

#[ic_cdk::query]
fn get_crop_insurance(id: u64) -> Result<CropInsurance, Error> {
    _get_crop_insurance(&id).ok_or(Error::NotFound {
        msg: format!("Crop insurance with id={} not found", id),
    })
}

#[ic_cdk::query]
fn get_insurance_claim(claim_id: u64) -> Result<InsuranceClaim, Error> {
    _get_insurance_claim(&claim_id).ok_or(Error::NotFound {
        msg: format!("Insurance claim with id={} not found", claim_id),
    })
}

#[ic_cdk::update]
fn add_debt(debt: DebtPayload) -> Result<Debt, Error> {
    // Validate input data
    validate_non_empty(&debt.debtor, "debtor")?;
    validate_non_empty(&debt.creditor, "creditor")?;
    if debt.amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Debt amount must be greater than zero".to_string(),
        });
    }

    let id = generate_unique_id()?;

    let debt = Debt {
        id,
        debtor: debt.debtor,
        creditor: debt.creditor,
        amount: debt.amount,
        created_at: time(),
    };

    do_insert_debt(&debt);
    Ok(debt)
}

#[ic_cdk::update]
fn update_debt(id: u64, payload: DebtPayload) -> Result<Debt, Error> {
    // Validate input data
    validate_non_empty(&payload.debtor, "debtor")?;
    validate_non_empty(&payload.creditor, "creditor")?;
    if payload.amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Debt amount must be greater than zero".to_string(),
        });
    }

    let mut debt = get_debt(id)?;
    debt.debtor = payload.debtor;
    debt.creditor = payload.creditor;
    debt.amount = payload.amount;

    do_insert_debt(&debt);
    Ok(debt)
}

#[ic_cdk::update]
fn create_escrow(payload: EscrowPayload) -> Result<Escrow, Error> {
    // Validate input data
    if payload.amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Escrow amount must be greater than zero".to_string(),
        });
    }

    // Ensure the corresponding debt exists
    let debt = get_debt(payload.debt_id)?;

    let escrow = Escrow {
        debt_id: debt.id,
        amount: payload.amount,
        created_at: time(),
    };

    do_insert_escrow(&escrow);
    Ok(escrow)
}

#[ic_cdk::update]
fn purchase_crop_insurance(payload: CropInsurancePayload) -> Result<CropInsurance, Error> {
    // Validate input data
    validate_non_empty(&payload.farmer, "farmer")?;
    validate_non_empty(&payload.crop_type, "crop_type")?;
    if payload.coverage_amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Coverage amount must be greater than zero".to_string(),
        });
    }

    let id = generate_unique_id()?;

    let insurance = CropInsurance {
        id,
        farmer: payload.farmer,
        crop_type: payload.crop_type,
        coverage_amount: payload.coverage_amount,
        coverage_start_date: payload.coverage_start_date,
        coverage_end_date: payload.coverage_end_date,
    };

    CROP_INSURANCE_STORAGE.with(|service| service.borrow_mut().insert(id, insurance.clone()));
    Ok(insurance)
}

#[ic_cdk::update]
fn submit_insurance_claim(payload: InsuranceClaimPayload) -> Result<InsuranceClaim, Error> {
    // Ensure the corresponding insurance exists
    let insurance = get_crop_insurance(payload.insurance_id)?;

    if payload.claim_amount == 0 {
        return Err(Error::InvalidInput {
            msg: "Claim amount must be greater than zero".to_string(),
        });
    }

    let claim_id = generate_unique_id()?;

    let claim = InsuranceClaim {
        insurance_id: insurance.id,
        claim_amount: payload.claim_amount,
        claim_date: time(),
    };

    INSURANCE_CLAIM_STORAGE.with(|service| service.borrow_mut().insert(claim_id, claim.clone()));
    Ok(claim)
}

fn do_insert_debt(debt: &Debt) {
    DEBT_STORAGE.with(|service| service.borrow_mut().insert(debt.id, debt.clone()));
}

fn do_insert_escrow(escrow: &Escrow) {
    ESCROW_STORAGE
        .with(|service| service.borrow_mut().insert(escrow.debt_id, escrow.clone()));
}

fn _get_debt(id: &u64) -> Option<Debt> {
    DEBT_STORAGE.with(|service| service.borrow().get(id))
}

fn _get_escrow(debt_id: &u64) -> Option<Escrow> {
    ESCROW_STORAGE.with(|service| service.borrow().get(debt_id))
}

fn _get_crop_insurance(id: &u64) -> Option<CropInsurance> {
    CROP_INSURANCE_STORAGE.with(|service| service.borrow().get(id))
}

fn _get_insurance_claim(claim_id: &u64) -> Option<InsuranceClaim> {
    INSURANCE_CLAIM_STORAGE.with(|service| service.borrow().get(claim_id))
}

ic_cdk::export_candid!();


type CropInsurance = record {
  id : nat64;
  coverage_amount : nat64;
  coverage_start_date : nat64;
  coverage_end_date : nat64;
  crop_type : text;
  farmer : text;
};
type CropInsurancePayload = record {
  coverage_amount : nat64;
  coverage_start_date : nat64;
  coverage_end_date : nat64;
  crop_type : text;
  farmer : text;
};
type Debt = record {
  id : nat64;
  debtor : text;
  created_at : nat64;
  amount : nat64;
  creditor : text;
};
type DebtPayload = record { debtor : text; amount : nat64; creditor : text };
type Error = variant {
  InvalidInput : record { msg : text };
  NotFound : record { msg : text };
  InternalError : record { msg : text };
};
type Escrow = record { debt_id : nat64; created_at : nat64; amount : nat64 };
type EscrowPayload = record { debt_id : nat64; amount : nat64 };
type InsuranceClaim = record {
  insurance_id : nat64;
  claim_date : nat64;
  claim_amount : nat64;
};
type InsuranceClaimPayload = record {
  insurance_id : nat64;
  claim_amount : nat64;
};
type Result = variant { Ok : Debt; Err : Error };
type Result_1 = variant { Ok : Escrow; Err : Error };
type Result_2 = variant { Ok : CropInsurance; Err : Error };
type Result_3 = variant { Ok : InsuranceClaim; Err : Error };
service : {
  add_debt : (DebtPayload) -> (Result);
  create_escrow : (EscrowPayload) -> (Result_1);
  get_crop_insurance : (nat64) -> (Result_2) query;
  get_debt : (nat64) -> (Result) query;
  get_escrow : (nat64) -> (Result_1) query;
  get_insurance_claim : (nat64) -> (Result_3) query;
  purchase_crop_insurance : (CropInsurancePayload) -> (Result_2);
  submit_insurance_claim : (InsuranceClaimPayload) -> (Result_3);
  update_debt : (nat64, DebtPayload) -> (Result);
}

type CropInsurance = record {
  id : nat64;
  coverage_amount : nat64;
  coverage_start_date : nat64;
  coverage_end_date : nat64;
  crop_type : text;
  farmer : text;
};
type CropInsurancePayload = record {
  coverage_amount : nat64;
  coverage_start_date : nat64;
  coverage_end_date : nat64;
  crop_type : text;
  farmer : text;
};
type Debt = record {
  id : nat64;
  debtor : text;
  created_at : nat64;
  amount : nat64;
  creditor : text;
};
type DebtPayload = record { debtor : text; amount : nat64; creditor : text };
type Error = variant {
  InvalidInput : record { msg : text };
  NotFound : record { msg : text };
  InternalError : record { msg : text };
};
type Escrow = record { debt_id : nat64; created_at : nat64; amount : nat64 };
type EscrowPayload = record { debt_id : nat64; amount : nat64 };
type InsuranceClaim = record {
  insurance_id : nat64;
  claim_date : nat64;
  claim_amount : nat64;
};
type InsuranceClaimPayload = record {
  insurance_id : nat64;
  claim_amount : nat64;
};
type Result = variant { Ok : Debt; Err : Error };
type Result_1 = variant { Ok : Escrow; Err : Error };
type Result_2 = variant { Ok : CropInsurance; Err : Error };
type Result_3 = variant { Ok : InsuranceClaim; Err : Error };
service : {
  add_debt : (DebtPayload) -> (Result);
  create_escrow : (EscrowPayload) -> (Result_1);
  get_crop_insurance : (nat64) -> (Result_2) query;
  get_debt : (nat64) -> (Result) query;
  get_escrow : (nat64) -> (Result_1) query;
  get_insurance_claim : (nat64) -> (Result_3) query;
  purchase_crop_insurance : (CropInsurancePayload) -> (Result_2);
  submit_insurance_claim : (InsuranceClaimPayload) -> (Result_3);
  update_debt : (nat64, DebtPayload) -> (Result);
}
