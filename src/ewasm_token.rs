// see ewasm "WRC20" pseudocode https://gist.github.com/axic/16158c5c88fbc7b1d09dfa8c658bc363
extern crate sha3;
use sha3::{Digest, Keccak256};

extern "C" {
    pub fn ethereum_getCallDataSize() -> u32;
    pub fn ethereum_callDataCopy(resultOffset: *const u32, dataOffset: u32, length: u32);
    pub fn ethereum_revert(dataOffset: *const u32, length: u32) -> !;
    pub fn ethereum_storageLoad(keyOffset: *const u32, resultOffset: *const u32);
    pub fn ethereum_storageStore(keyOffset: *const u32, valueOffset: *const u32);
    pub fn ethereum_finish(dataOffset: *const u32, length: u32) -> !;
    pub fn ethereum_getCaller(resultOffset: *const u32);
}

fn calculate_allowance_hash(sender: &[u8;20], spender: &[u8;20]) -> Vec<u8> {
    let mut allowance: Vec<u8> = vec![97, 108, 108, 111, 119, 97, 110, 99, 101]; // "allowance"
    allowance.extend_from_slice(sender);
    allowance.extend_from_slice(spender);

    let mut hasher = Keccak256::default();
    hasher.input(allowance);

    hasher.result().as_slice().to_owned()
}

fn calculate_balance_hash(address: &[u8;20]) -> Vec<u8> {
    let mut balanceOf: Vec<u8> = vec![98,  97, 108,  97, 110, 99, 101, 79, 102]; // "balanceOf"
    balanceOf.extend_from_slice(address);

    let mut hasher = Keccak256::default();
    hasher.input(balanceOf);
    
    hasher.result().as_slice().to_owned()
}

fn get_balance(address: &[u8;20]) -> [u8;32]{
    let storage_key = calculate_balance_hash(&address);

    let mut balance: [u8;32] = [0;32];

    unsafe {
        ethereum_storageLoad(storage_key.as_ptr() as *const u32, balance.as_mut_ptr() as *const u32);
    }

    return balance;
    
}

fn set_balance(address: &[u8;20], value: &[u8;32]) {
    let storage_key = calculate_balance_hash(&address);

    unsafe {
        ethereum_storageStore(storage_key.as_ptr() as *const u32, value.as_ptr() as *const u32);
    }
}

fn get_allowance(sender: &[u8;20], &spender: &[u8;20]) -> [u8;32] {
    let allowance_key = calculate_allowance_hash(&sender, &spender);

    let mut allowed_value: [u8;32] = [0;32];

    unsafe {
        ethereum_storageLoad(allowance_key.as_ptr() as *const u32 , allowed_value.as_mut_ptr() as *const u32);
    }

    return allowed_value;
}

fn set_allowance(sender: &[u8;20], spender: &[u8;20], value: &[u8;32]) {
    let allowance_key = calculate_allowance_hash(&sender, &spender);

    unsafe {
        ethereum_storageStore(allowance_key.as_ptr() as *const u32, value.as_ptr() as *const u32);
    }
}

#[no_mangle]
pub fn main() {

    let decimals: u64 = 0;
    
    let total_supply: u64 = 100000000;

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

    let data_size: u32;
    unsafe {
        data_size = ethereum_getCallDataSize();
    }

    if data_size < 4 {
        unsafe {
            ethereum_revert(0 as *const u32, 0 as u32);
        }
    }
    
    let function_selector: [u8;4] = [0;4];
    unsafe {
        ethereum_callDataCopy(function_selector.as_ptr() as *const u32, 0, 4);
    }

    // DO BALANCE
    if function_selector == do_balance_signature {
        if data_size != 24 {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        let address: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(address.as_ptr() as *const u32, 4, 20);
        }

        let balance = get_balance(&address);

        // checks balance is not 0
        let blank: [u8; 32] = [0;32];
        if balance != blank {
            unsafe {
                ethereum_finish(balance.as_ptr() as *const u32, balance.len() as u32);
            }
        }
    }

    // DO TRANSFER
    if function_selector == do_transfer_signature {
        if data_size != 32 {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        // Get Sender
        let mut sender: [u8; 20] = [0;20];
        unsafe {
            ethereum_getCaller(sender.as_mut_ptr() as *const u32);
        }

        // Get Recipient
        let mut recipient: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(recipient.as_mut_ptr() as *const u32, 4, 20);
        }

        // Get Value
        let mut value_data: [u8;8] = [0;8];
        unsafe {
            ethereum_callDataCopy(value_data.as_mut_ptr() as *const u32, 24, 8);
        }
        let mut value: [u8; 32] = [0;32];
        value[24..].copy_from_slice(&value_data[0..8]);

        // Get Sender Balance
        let sender_balance = get_balance(&sender);

        // Get Recipient Balance
        let recipient_balance = get_balance(&recipient);

        // Substract sender balance
        let mut sb_bytes: [u8; 8] = [0;8];
        sb_bytes.copy_from_slice(&sender_balance[24..32]);
        let sb_u64 = u64::from_be_bytes(sb_bytes);

        let mut val_bytes: [u8; 8] = [0;8];
        val_bytes.copy_from_slice(&value[24..32]);
        let val_u64 = u64::from_be_bytes(val_bytes);

        let new_sb_u64 = sb_u64 - val_u64;

        let mut sb_value: [u8; 32] = [0;32];
        let mut new_sb_bytes: [u8;8] = [0;8];
        
        new_sb_bytes = new_sb_u64.to_be_bytes();

        sb_value[24..32].copy_from_slice(&new_sb_bytes[0..8]);
        
        // Adds recipient balance
        let mut rc_bytes: [u8; 8] = [0;8];
        rc_bytes.copy_from_slice(&recipient_balance[24..32]);
        let rc_u64 = u64::from_be_bytes(rc_bytes);

        let new_rc_u64 = rc_u64 + val_u64;
        
        let mut rc_value: [u8; 32] = [0;32];
        let mut new_rc_bytes: [u8; 8] = [0;8];
        new_rc_bytes = new_rc_u64.to_be_bytes();

        rc_value[24..32].copy_from_slice(&new_rc_bytes[0..8]);

        set_balance(&sender, &sb_value);
        set_balance(&recipient, &rc_value);
        
    }

    // NAME
    if function_selector == name_signature {
        let token_name = "EwasmCoin";
        unsafe {
            ethereum_finish(token_name.as_ptr() as *const u32, token_name.len() as u32);
        }
    }

    // SYMBOL
    if function_selector == symbol_signature {
        let symbol = "EWC";
        unsafe {
            ethereum_finish(symbol.as_ptr() as *const u32, symbol.len() as u32);
        }
    }

    // DECIMALS
    if function_selector == decimals_signature {
        let decimals_ptr: *const u64 = &decimals;
        unsafe {
            ethereum_finish(decimals_ptr as *const u32, 8 as u32);
        }
    }

    // TOTALSUPPLY
    if function_selector == total_supply_signature {
        let total_supply_ptr: *const u64 = &total_supply;
        unsafe {
            ethereum_finish(total_supply_ptr as *const u32, 8 as u32);
        }
    }
    
    // APPROVE
    if function_selector == approve_signature {
        // allowance[sender][spender] = _value

        let mut sender: [u8; 20] = [0;20];
        unsafe {
            ethereum_getCaller(sender.as_mut_ptr() as *const u32);
        }

        let mut spender: [u8;20] = [0;20];
        unsafe {
            ethereum_callDataCopy(spender.as_mut_ptr() as *const u32, 4, 20);
        }

        let mut value: [u8;8] = [0;8];
        unsafe {
            ethereum_callDataCopy(value.as_mut_ptr() as *const u32, 24, 8);
        }

        let mut storage_value: [u8;32] = [0;32];
        storage_value[24..32].copy_from_slice(&value[0..8]);
        set_allowance(&sender, &spender, &storage_value);

    }

    // ALLOWANCE
    if function_selector == allowance_signature {
        if data_size != 44 {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }
        
        let mut from: [u8;20] = [0;20];
        unsafe {
            ethereum_callDataCopy(from.as_mut_ptr() as *const u32, 4, 24);
        }

        let mut spender: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(spender.as_mut_ptr() as *const u32, 24, 20);
        }

        let allowance_value = get_allowance(&from, &spender);

        unsafe {
            ethereum_finish(allowance_value.as_ptr() as *const u32, 32 as u32);
        }
        
    }

    // TRANSFERFROM
    if function_selector == transfer_from_signature {
        // validate input data
        if data_size != 52 {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        // get sender
        let mut sender: [u8; 20] = [0; 20];
        unsafe {
            ethereum_getCaller(sender.as_mut_ptr() as *const u32);
        }
        
        // get owner
        let mut owner: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(owner.as_mut_ptr() as *const u32, 4, 20);
        }

        // get recipient
        let mut recipient:[u8;20] = [0;20];
        unsafe {
            ethereum_callDataCopy(recipient.as_mut_ptr() as *const u32, 24, 20);
        }
        
        // get value
        let mut value_data: [u8;8] = [0;8];
        unsafe {
            ethereum_callDataCopy(value_data.as_mut_ptr() as *const u32, 44, 8);
        }

        let value = u64::from_be_bytes(value_data);

        // get owner balance
        let owner_balance = get_balance(&owner);

        let mut ob_bytes: [u8;8] = [0;8];
        ob_bytes.copy_from_slice(&owner_balance[24..32]);
        let mut owner_balance = u64::from_be_bytes(ob_bytes);

        let return_ptr: *const u8 = function_selector.as_ptr();
        // owner has enough funds
        if owner_balance < value {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        // sender has authorization to transfer funds
        let mut allowed_value = get_allowance(&owner, &sender);

        let mut a_bytes:[u8;8] = [0;8];
        a_bytes.copy_from_slice(&allowed_value[24..32]);
        let mut allowed = u64::from_be_bytes(a_bytes);

        if value > allowed {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        // get recipient balance
        let recipient_balance = get_balance(&recipient);

        let mut rb_bytes: [u8;8] = [0;8];
        rb_bytes.copy_from_slice(&recipient_balance[24..32]);
        let mut recipient_balance = u64::from_be_bytes(rb_bytes);

        // transfer:
        owner_balance = owner_balance - value;
        recipient_balance = recipient_balance + value;
        allowed = allowed - value;

        let mut stv_owner_balance: [u8;32] = [0;32]; // value to be stored (owner)
        let mut owner_balance_bytes: [u8;8] = [0;8]; // u64 converted to bytes
        owner_balance_bytes = owner_balance.to_be_bytes();
        stv_owner_balance[24..32].copy_from_slice(&owner_balance_bytes[0..8]);
        
        let mut stv_recipient_balance: [u8;32] = [0;32]; // value to be stored (recipient)
        let mut recipient_balance_bytes: [u8;8] = [0;8]; // u64 converted to bytes
        recipient_balance_bytes = recipient_balance.to_be_bytes();
        stv_recipient_balance[24..32].copy_from_slice(&recipient_balance_bytes[0..8]);

        let mut stv_allowed: [u8;32] = [0;32]; // value to be stored (allowance)
        let mut allowed_bytes: [u8;8] = [0;8]; // u64 converted to bytes
        allowed_bytes = allowed.to_be_bytes();
        stv_allowed[24..32].copy_from_slice(&allowed_bytes[0..8]);

        set_balance(&owner, &stv_owner_balance);
        set_balance(&recipient, &stv_recipient_balance);
        set_allowance(&owner, &sender, &stv_allowed);

    }
    
    return;
}
