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

#[no_mangle]
pub fn main() {

    let decimals = 0;
    
    let total_supply: u32 = 100000000;
                       // 0x73  38   95  4a  68    f7  2e   8a  53   d4  3a  68   35  ee   45  cd   5f  01 4c   76
    let owner: [u8; 20] = [115, 56, 149, 74, 104, 247, 46, 138, 83, 212, 58, 104, 53, 238, 69, 205, 95, 1, 76, 118 ];
    
    // 0x9993021a do_balance() ABI signature
    let do_balance_signature: [u8; 4] = [153, 147, 2, 26];

    // 0x5d359fbd do_transfer() ABI signature
    let do_transfer_signature: [u8; 4] = [93, 53, 159, 189];

    // 0x06fdde03 name() ABI signature
    let name_signature: [u8;4] = [6, 253, 222, 3];

    // 0x95d89b41 symbol() ABI signature
    let symbol_signature: [u8;4] = [149, 216, 155, 65];
    
    // 0x1086a9aa approve() ABI signature
    let approve_signature: [u8; 4] = [16, 134, 169, 170];
    

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

        let balanceOf: [u8;9] = [98, 97, 108, 97, 110, 99, 101, 79, 102]; // "balanceOf"

        let address: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(address.as_ptr() as *const u32, 4, 20);
        }

        let mut pre_storage_key: Vec<u8> = balanceOf.to_vec();
        pre_storage_key.extend_from_slice(&address);

        let mut hasher = Keccak256::default();
        hasher.input(pre_storage_key);
        let storage_key = hasher.result();

        let mut balance: [u8;32] = [0;32];

        unsafe {
            ethereum_storageLoad(storage_key.as_ptr() as *const u32, balance.as_mut_ptr() as *const u32);
        }

        // checks the balance is not 0
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

        let balanceOf: [u8;9] = [98, 97, 108, 97, 110, 99, 101, 79, 102]; // "balanceOf"

        // Get Sender
        let mut sender: [u8; 20] = [0;20];
        unsafe {
            ethereum_getCaller(sender.as_mut_ptr() as *const u32);
        }
        
        let mut pre_sender_key: Vec<u8> = balanceOf.to_vec();
        pre_sender_key.extend_from_slice(&sender);

        let mut hasher = Keccak256::default();
        hasher.input(pre_sender_key);
        let sender_key = hasher.result();

        // Get Recipient
        let mut recipient_data: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(recipient_data.as_mut_ptr() as *const u32, 4, 20);
        }

        let mut pre_recipient_key: Vec<u8> = balanceOf.to_vec();
        pre_recipient_key.extend_from_slice(&recipient_data);

        let mut hasher = Keccak256::default();
        hasher.input(pre_recipient_key);
        let recipient_key = hasher.result();

        // Get Value
        let mut value_data: [u8;8] = [0;8];
        unsafe {
            ethereum_callDataCopy(value_data.as_mut_ptr() as *const u32, 24, 8);
        }
        let mut value: [u8; 32] = [0;32];
        value[24..].copy_from_slice(&value_data[0..8]);

        // Get Sender Balance
        let mut sender_balance: [u8; 32] = [0;32];
        unsafe {
            ethereum_storageLoad(sender_key.as_ptr() as *const u32, sender_balance.as_mut_ptr() as *const u32);
        }

        // Get Recipient Balance
        let mut recipient_balance: [u8;32] = [0;32];
        unsafe {
            ethereum_storageLoad(recipient_key.as_ptr() as *const u32, recipient_balance.as_mut_ptr() as *const u32);
        }

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

        // save new sender balance
        unsafe {
            ethereum_storageStore(sender_key.as_ptr() as *const u32, sb_value.as_ptr() as *const u32);
        }

        // save recipient balance
        unsafe {
            ethereum_storageStore(recipient_key.as_ptr() as *const u32, rc_value.as_ptr() as *const u32);
        }
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
    
    // APPROVE
    if function_selector == approve_signature {
        // "allowance".sender.spender = _value
        let allowance: [u8; 9] = [97, 108, 108, 111, 119, 97, 110, 99, 101]; // "allowance"

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

        // concatenation: "allowance".sender.spender
        let mut key: Vec<u8> = allowance.to_vec();
        key.extend_from_slice(&sender);
        key.extend_from_slice(&spender);

        let mut hasher = Keccak256::default();

        hasher.input(key);

        // use hash of concatenation as storage key
        let storage_key = hasher.result();

        // store value
        let mut storage_value: [u8;32] = [0;32];
        storage_value[24..32].copy_from_slice(&value[0..8]);

        unsafe {
            ethereum_storageStore(storage_key.as_ptr() as *const u32, storage_value.as_ptr() as *const u32);
        }
    }

    // ALLOWANCE

    // TRANSFERFROM

    // TOTALSUPPLY
    
    return;
}
