// see ewasm "WRC20" pseudocode https://gist.github.com/axic/16158c5c88fbc7b1d09dfa8c658bc363

use std::mem::transmute;

extern "C" {
    pub fn ethereum_getCallDataSize() -> u32;
    pub fn ethereum_callDataCopy(resultOffset: *const u32, dataOffset: u32, length: u32);
    pub fn ethereum_revert(dataOffset: *const u32, length: u32) -> !;
    pub fn ethereum_storageLoad(keyOffset: *const u32, resultOffset: *const u32);
    pub fn ethereum_storageStore(keyOffset: *const u32, valueOffset: *const u32);
    pub fn ethereum_finish(dataOffset: *const u32, length: u32) -> !;
    pub fn ethereum_getCaller(resultOffset: *const u32);
}


fn from_be_bytes(bytes: [u8;8]) -> u64 {
    let mut result: u64 = 0;
    unsafe {
        result = transmute::<[u8;8], u64>(bytes);
    }
    return result;
}

fn to_be_bytes(number: u64) -> [u8;8]{
    let mut result: [u8;8] = [0;8];
    unsafe {
        result = transmute::<u64, [u8;8]>(number);
    }
    return result;
}

#[no_mangle]
pub fn main() {
    // 0x9993021a do_balance() ABI signature
    let do_balance_signature: [u8; 4] = [153, 147, 2, 26];

    // 0x5d359fbd do_transfer() ABI signature
    let do_transfer_signature: [u8; 4] = [93, 53, 159, 189];
    

    let data_size: u32;
    unsafe {
        data_size = ethereum_getCallDataSize();
    }

    /*
    let mut input_data: Vec<u8> = Vec::with_capacity(data_size as usize);
    unsafe {
        input_data.set_len(data_size as usize);
    }
    
    unsafe {
        ethereum_callDataCopy(input_data.as_mut_ptr() as *const u32, // resultOffset
                              0 as u32,                              // dataOffset
                              data_size as u32);                     // length
    }
    */

    if data_size < 4 {
        unsafe {
            ethereum_revert(0 as *const u32, 0 as u32);
        }
    }
    
    //let function_selector = input_data[0..4].to_vec();
    let function_selector: [u8;4] = [0;4];
    unsafe {
        ethereum_callDataCopy(function_selector.as_ptr() as *const u32, 0, 4);
    }

    if function_selector == do_balance_signature {

        if data_size != 24 {
            unsafe {
                ethereum_revert(0 as *const u32, 0 as u32);
            }
        }

        //let address = input_data[4..].to_vec();
        let address: [u8; 20] = [0;20];
        unsafe {
            ethereum_callDataCopy(address.as_ptr() as *const u32, 4, 20);
        }

        let mut storage_key: [u8; 32] = [0;32];

        storage_key[12..].copy_from_slice(&address[0..20]);
            
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
        
        let mut sender_key: [u8; 32] = [0;32];

        sender_key[12..].copy_from_slice(&sender[0..20]);

        // Get Recipient
        //let recipient_data = input_data[4..24].to_vec();
        let mut recipient_key: [u8; 32] = [0;32];
        unsafe {
            ethereum_callDataCopy(recipient_key.as_mut_ptr() as *const u32, 4, 20);
        }
        
        //recipient_key[12..].copy_from_slice(&recipient_data[..]);

        // Get Value
        //let value_data = input_data[24..].to_vec();
        let mut value_data: [u8;8] = [0;8];
        unsafe {
            ethereum_callDataCopy(value_data.as_mut_ptr() as *const u32, 24, 8);
        }
        let mut value: [u8; 32] = [0;32];
        value[24] = value_data[0];
        value[25] = value_data[1];
        value[26] = value_data[2];
        value[27] = value_data[3];
        value[28] = value_data[4];
        value[29] = value_data[5];
        value[20] = value_data[6];
        value[31] = value_data[7];

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
        //let sb_u64 = u64::from_be_bytes(sb_bytes);
        let sb_u64 = from_be_bytes(sb_bytes);

        let mut val_bytes: [u8; 8] = [0;8];
        val_bytes.copy_from_slice(&value[24..32]);
        //let val_u64 = u64::from_be_bytes(val_bytes);
        let val_u64 = from_be_bytes(val_bytes);

        let new_sb_u64 = sb_u64 - val_u64;

        let mut sb_value: [u8; 32] = [0;32];
        let mut new_sb_bytes: [u8;8] = [0;8];
        
        new_sb_bytes = to_be_bytes(new_sb_u64);

        sb_value[24..32].copy_from_slice(&new_sb_bytes[0..8]);
        
        // Adds recipient balance
        let mut rc_bytes: [u8; 8] = [0;8];
        rc_bytes.copy_from_slice(&recipient_balance[24..32]);
        //let rc_u64 = u64::from_be_bytes(rc_bytes);
        let rc_u64 = from_be_bytes(rc_bytes);

        let new_rc_u64 = rc_u64 + val_u64;
        
        let mut rc_value: [u8; 32] = [0;32];
        let mut new_rc_bytes: [u8; 8] = [0;8];
        new_rc_bytes = to_be_bytes(new_rc_u64);

        rc_value[24..32].copy_from_slice(&new_rc_bytes[0..8]);

        unsafe {
            ethereum_storageStore(sender_key.as_ptr() as *const u32, sb_value.as_ptr() as *const u32);
        }
        
        unsafe {
            ethereum_storageStore(recipient_key.as_ptr() as *const u32, rc_value.as_ptr() as *const u32);
        }
    } 
    return;
}
