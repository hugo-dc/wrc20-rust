// see ewasm "WRC20" pseudocode https://gist.github.com/axic/16158c5c88fbc7b1d09dfa8c658bc363
extern crate ewasm_api;
extern crate sha3;
use ewasm_api::types::*;
use sha3::{Digest, Keccak256};


fn calculate_allowance_hash(sender: &[u8;20], spender: &[u8;20]) -> Vec<u8>{
    let mut allowance: Vec<u8> = vec![97, 108, 108, 111, 119, 97, 110, 99, 101]; // "allowance"
    allowance.extend_from_slice(sender);
    allowance.extend_from_slice(spender);

    let mut hasher = Keccak256::default();
    hasher.input(allowance);

    hasher.result().as_slice().to_owned()
}
// TODO: should return StorageKey
fn calculate_balance_hash(address: &[u8;20]) -> Vec<u8> {
    let mut balanceOf: Vec<u8> = vec![98,  97, 108,  97, 110, 99, 101, 79, 102]; // "balanceOf"
    balanceOf.extend_from_slice(address);

    let mut hasher = Keccak256::default();
    hasher.input(balanceOf);
    
    hasher.result().as_slice().to_owned()
}

fn get_balance(address: &Address) -> StorageValue {
    let hash = calculate_balance_hash(&address.bytes);
    
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    
    //let balance = ewasm_api::storage_load(
    let balance = ewasm_api::storage_load(&storage_key);

    return balance;
}

fn set_balance(address: &Address, value: &StorageValue) {
    let hash = calculate_balance_hash(&address.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
}

fn get_allowance(sender: &Address, spender: &Address) -> StorageValue {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);
    
    ewasm_api::storage_load(&storage_key)
}

fn set_allowance(sender: &Address, spender: &Address, value: &StorageValue) {
    let hash = calculate_allowance_hash(&sender.bytes, &spender.bytes);
    let mut storage_key = StorageKey::default();
    storage_key.bytes.copy_from_slice(&hash[0..32]);

    ewasm_api::storage_store(&storage_key, &value);
    
}

#[no_mangle]
pub fn main() {
    // 0x9993021a do_balance() ABI signature
    let do_balance_signature: [u8; 4] = [153, 147, 2, 26];

    // 0x5d359fbd do_transfer() ABI signature
    let do_transfer_signature: [u8; 4] = [93, 53, 159, 189];
    
    // 0x06fdde03 name() ABI signature
    let name_signature: [u8;4] = [6, 253, 222, 3];

    // 0x95d89b41 symbol() ABI signature
    let symbol_signature: [u8;4] = [149, 216, 155, 65];

    // 0x313ce567 decimals() ABI signature
    let decimals_signature: [u8;4] = [49, 60, 229, 103];

    // 0x18160ddd totalSupply() ABI signature
    let total_supply_signature: [u8;4] = [24, 22, 13, 221];

    // 0x1086a9aa approve(address, uint64) ABI signature
    let approve_signature: [u8; 4] = [16, 134, 169, 170];

    // 0xdd62ed3e allowance(address, address) ABI signature
    let allowance_signature: [u8; 4] = [221, 98, 237, 62];

    // 0x2ea0dfe1 transferFrom(address,address,uint64) ABI signature
    let transfer_from_signature: [u8; 4] = [46, 160, 223, 225];

    let data_size = ewasm_api::calldata_size();
    let input_data = ewasm_api::calldata_acquire();

    if data_size < 4 {
        ewasm_api::revert();
    }
    
    let function_selector = input_data[0..4].to_vec();

    // DO BALANCE
    if function_selector == do_balance_signature {

        if data_size != 24 {
            ewasm_api::revert();
        }

        let address_data = input_data[4..].to_vec();
        let mut address = Address::default();
        address.bytes.copy_from_slice(&address_data[0..20]);

        let balance = get_balance(&address);

        // checks the balance is not 0
        if balance.bytes != StorageValue::default().bytes {
            ewasm_api::finish_data(&balance.bytes);
        }
    }


    if function_selector == do_transfer_signature {
        if input_data.len() != 32 {
            ewasm_api::revert();
        }

        // Get Sender
        let sender = ewasm_api::caller();

        // Get Recipient
        let recipient_data = input_data[4..24].to_vec();
        let mut recipient = Address::default();
        recipient.bytes.copy_from_slice(&recipient_data[0..20]);
        
        // Get Value
        let mut value_data:[u8;8] = [0;8];
        value_data.copy_from_slice(&input_data[24..]);
        let mut value = StorageValue::default();
        let value_len = value_data.len();
        let start = 32 - value_len;

        for i in start..(value_len+start) {
            value.bytes[i] = value_data[i-start];
        }

        // Get Sender Balance
        let sender_balance = get_balance(&sender);

        // Get Recipient Balance
        let recipient_balance = get_balance(&recipient);

        // Substract sender balance
        let mut sb_bytes: [u8; 8] = Default::default();
        sb_bytes.copy_from_slice(&sender_balance.bytes[24..32]);
        let sb_u64 = u64::from_be_bytes(sb_bytes);

        let mut val_bytes: [u8; 8] = Default::default();
        val_bytes.copy_from_slice(&value.bytes[24..32]);
        let val_u64 = u64::from_be_bytes(val_bytes);

        let new_sb_u64 = sb_u64 - val_u64;

        let mut sb_value = StorageValue::default();
        let mut new_sb_bytes: [u8;8] = Default::default();
        new_sb_bytes = new_sb_u64.to_be_bytes();

        sb_value.bytes[24..32].copy_from_slice(&new_sb_bytes[0..8]);
        
        // Adds recipient balance
        let mut rc_bytes: [u8; 8] = Default::default();
        rc_bytes.copy_from_slice(&recipient_balance.bytes[24..32]);
        let rc_u64 = u64::from_be_bytes(rc_bytes);

        let new_rc_u64 = rc_u64 + val_u64;
        
        let mut rc_value = StorageValue::default();
        let mut new_rc_bytes: [u8; 8] = Default::default();
        new_rc_bytes = new_rc_u64.to_be_bytes();

        rc_value.bytes[24..32].copy_from_slice(&new_rc_bytes[0..8]);

        set_balance(&sender, &sb_value);
        set_balance(&recipient, &rc_value);

    }

    // NAME
    if function_selector == name_signature {
        let token_name = "EwasmCoin".to_string().into_bytes();
        ewasm_api::finish_data(&token_name);
    }

    // SYMBOL
    if function_selector == symbol_signature {
        let symbol = "EWC".to_string().into_bytes();
        ewasm_api::finish_data(&symbol);
    }

    // DECIMALS
    if function_selector == decimals_signature {
        let decimals: u64 = 0;
        let decimals_bytes = decimals.to_be_bytes();
        ewasm_api::finish_data(&decimals_bytes);
    }

    // TOTALSUPPLY
    if function_selector == total_supply_signature {
        let total_supply: u64 = 100000000;
        let total_supply_bytes = total_supply.to_be_bytes();
        ewasm_api::finish_data(&total_supply_bytes);
    }

    // APPROVE
    if function_selector == approve_signature {
        // allowance[sender][spender] = _value

        let sender = ewasm_api::caller();

        let spender_data = input_data[4..24].to_vec();
        let mut spender = Address::default();
        spender.bytes.copy_from_slice(&spender_data[0..20]);


        let value = input_data[24..32].to_vec();

        let mut storage_value = StorageValue::default();
        storage_value.bytes[24..32].copy_from_slice(&value[0..8]);
        set_allowance(&sender, &spender, &storage_value);
        
    }

    // ALLOWANCE
    if function_selector == allowance_signature {
        if data_size != 44 {
            ewasm_api::revert();
        }

        let from_data = input_data[4..24].to_vec();
        let mut from = Address::default();
        from.bytes.copy_from_slice(&from_data[0..20]); 

        let spender_data = input_data[24..44].to_vec();
        let mut spender = Address::default();
        spender.bytes.copy_from_slice(&spender_data[0..20]); 

        let allowance_value = get_allowance(&from, &spender);

        ewasm_api::finish_data(&allowance_value.bytes);
    }

    // TRANSFERFROM
    if function_selector == transfer_from_signature {
        if data_size != 52 {
            ewasm_api::revert();
        }

        let sender = ewasm_api::caller();

        let mut owner = Address::default();
        owner.bytes.copy_from_slice(&input_data[4..24]);

        let mut recipient = Address::default();
        recipient.bytes.copy_from_slice(&input_data[24..44]);

        let mut value_data: [u8;8] = [0;8];
        value_data.copy_from_slice(&input_data[44..52]);

        let value = u64::from_be_bytes(value_data);

        // get owner_balance
        let owner_balance = get_balance(&owner);

        let mut ob_bytes: [u8;8] = [0;8];
        ob_bytes.copy_from_slice(&owner_balance.bytes[24..32]);
        let mut owner_balance = u64::from_be_bytes(ob_bytes);

        if owner_balance < value {
            ewasm_api::revert();
        }

        // sender has authorization to transfer funds
        let mut allowed_value = get_allowance(&owner, &sender);

        let mut a_bytes: [u8;8] = [0;8];
        a_bytes.copy_from_slice(&allowed_value.bytes[24..32]);
        let mut allowed = u64::from_be_bytes(a_bytes);

        if value > allowed {
            ewasm_api::revert();
        }

        // get recipient balance
        let recipient_balance = get_balance(&recipient);

        let mut rb_bytes: [u8;8] = [0;8];
        rb_bytes.copy_from_slice(&recipient_balance.bytes[24..32]);
        let mut recipient_balance = u64::from_be_bytes(rb_bytes);

        // transfer
        owner_balance = owner_balance - value;
        recipient_balance = recipient_balance + value;
        allowed = allowed - value;

        let mut stv_owner_balance = StorageValue::default();
        let mut owner_balance_bytes: [u8;8] = [0;8];
        owner_balance_bytes = owner_balance.to_be_bytes();
        stv_owner_balance.bytes[24..32].copy_from_slice(&owner_balance_bytes[0..8]);

        let mut stv_recipient_balance = StorageValue::default();
        let mut recipient_balance_bytes: [u8;8] = [0;8];
        recipient_balance_bytes = recipient_balance.to_be_bytes();
        stv_recipient_balance.bytes[24..32].copy_from_slice(&recipient_balance_bytes[0..8]);

        let mut stv_allowed = StorageValue::default();
        let mut allowed_bytes: [u8;8] = [0;8];
        allowed_bytes = allowed.to_be_bytes();
        stv_allowed.bytes[24..32].copy_from_slice(&allowed_bytes[0..8]);

        set_balance(&owner, &stv_owner_balance);
        set_balance(&recipient, &stv_recipient_balance);
        set_allowance(&owner, &sender, &stv_allowed);
        
    }
    
    return;
}
