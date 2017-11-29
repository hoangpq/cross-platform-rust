// Copyright 2016 Mozilla
//
// Licensed under the Apache License, Version 2.0 (the "License"); you may not use
// this file except in compliance with the License. You may obtain a copy of the
// License at http://www.apache.org/licenses/LICENSE-2.0
// Unless required by applicable law or agreed to in writing, software distributed
// under the License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR
// CONDITIONS OF ANY KIND, either express or implied. See the License for the
// specific language governing permissions and limitations under the License.

use libc::size_t;
use std::os::raw::{
    c_char,
    c_int,
};
use std::ptr;

use time::Timespec;

use ffi_utils::strings::{
    string_to_c_char,
    c_char_to_string,
    to_string
};
use ffi_utils::android::log;
use labels::Label;

#[derive(Debug, Clone)]
pub struct Item {
    pub uuid: String,
    pub name: String,
    pub due_date: Option<Timespec>,
    pub completion_date: Option<Timespec>,
    pub labels: Vec<Label>,
}

impl Drop for Item {
    fn drop(&mut self) {
        println!("{:?} is being deallocated", self);
    }
}

pub fn new_item() -> Item {
    Item {
        uuid: "".to_string(),
        name: "".to_string(),
        due_date: None,
        completion_date: None,
        labels: vec![]
    }
}

#[no_mangle]
pub extern "C" fn item_new() -> *mut Item {
    let item = new_item();
    let boxed_item = Box::new(item);
    Box::into_raw(boxed_item)
}

#[no_mangle]
pub unsafe extern "C" fn item_destroy(item: *mut Item) {
    let _ = Box::from_raw(item);
}

// TODO Can these simpler android methods also work for swift?
#[no_mangle]
pub extern fn a_item_new() -> Box<Item> {
    Box::new(new_item())
}

pub extern "C" fn a_item_destroy(_: Box<Item>) {
    // Rust will clean up for us automatically, since we own the Item.
}

#[no_mangle]
pub unsafe extern "C" fn item_get_name(item: *const Item) -> *mut c_char {
    let item = &*item;
    string_to_c_char(item.name.clone())
}

#[no_mangle]
pub unsafe extern "C" fn item_set_name(item: *mut Item, name: *const c_char) {
    let item = &mut*item;
    item.name = c_char_to_string(name);
}

#[no_mangle]
pub unsafe extern "C" fn a_item_set_name(item: &mut Item, name: *const c_char) {
    log(&format!("NAME: Got item: {:?}", item)[..]);
    item.name = to_string(name);
    log(&format!("NAME: Updated item: {:?}", item)[..]);
}

#[no_mangle]
pub unsafe extern "C" fn item_get_due_date(item: *const Item) -> *mut i64 {
    let item = &*item;
    match item.due_date {
        Some(date) => {
            println!("item_get_due_date: returning {:?} for {:?}", date.sec, item.name);
            Box::into_raw(Box::new(date.sec))
        },
        None => {
            println!("item_get_due_date: returning null_mut for {:?}", item.name);
            ptr::null_mut()
        }
    }

}

#[no_mangle]
pub unsafe extern "C" fn item_set_due_date(item: *mut Item, due_date: *const size_t) {
    let item = &mut*item;
    log(&format!("DUE DATE: Got item: {:?}", item)[..]);
    if !due_date.is_null() {
        item.due_date = Some(Timespec::new(due_date as i64, 0));
    } else {
        item.due_date = None;
    }
    log(&format!("DUE DATE: Updated item: {:?}", item)[..]);
}

#[no_mangle]
pub unsafe extern "C" fn a_item_set_due_date(item: &mut Item, due_date: *const size_t) {
    log(&format!("DUE DATE: Got item: {:?}", item)[..]);
    if !due_date.is_null() {
        item.due_date = Some(Timespec::new(due_date as i64, 0));
    } else {
        item.due_date = None;
    }
    log(&format!("DUE DATE: Updated item: {:?}", item)[..]);
}

#[no_mangle]
pub unsafe extern "C" fn item_get_completion_date(item: *const Item) -> *mut i64 {
    let item = &*item;
    match item.completion_date {
        Some(date) => {
            println!("item_get_due_date: returning {:?} for {:?}", date.sec, item.name);
            Box::into_raw(Box::new(date.sec))
        },
        None => {
            println!("item_get_due_date: returning null_mut for {:?}", item.name);
            ptr::null_mut()
        }
    }

}

#[no_mangle]
pub unsafe extern "C" fn item_set_completion_date(item: *mut Item, completion_date: *const size_t) {
    let item = &mut*item;
    if !completion_date.is_null() {
        item.completion_date = Some(Timespec::new(completion_date as i64, 0));
    } else {
        item.completion_date = None;
    }
}

#[no_mangle]
pub unsafe extern "C" fn item_get_labels(item: *const Item) -> *mut Vec<Label> {
    let item = &*item;
    let boxed_labels = Box::new(item.labels.clone());
    Box::into_raw(boxed_labels)
}

#[no_mangle]
pub unsafe extern "C" fn item_labels_count(item: *const Item) -> c_int {
    let item = &*item;
    item.labels.len() as c_int
}

#[no_mangle]
pub unsafe extern "C" fn item_label_at(label_list: *const Vec<Label>, index: size_t) -> *const Label {
    let label_list = &*label_list;
    let index = index as usize;
    let label = Box::new(label_list[index].clone());
    Box::into_raw(label)
}

use std::panic;
use std::any::Any;

#[cfg(target_os="android")]
#[allow(non_snake_case)]
pub mod android {
    extern crate jni;

    use super::*;
    use self::jni::JNIEnv;
    use self::jni::objects::{JClass, JString, JValue};
    use self::jni::sys::{jlong};
    use ListManager;

    // #[no_mangle]
    // pub unsafe extern fn Java_com_mozilla_toodle_Item_newItem(env: JNIEnv, _: JClass) -> jlong {
    //     Box::into_raw(Box::new(new_item())) as jlong
    // }

    // #[no_mangle]
    // pub unsafe extern fn Java_com_mozilla_toodle_Item_itemSetName(env: JNIEnv, class: JClass, item: *mut Item, name: JString) {
    //     // debugging notes:
    //     // still not certain why this doesn't work as you'd expect it to. but here's a curious thing:
    //     // if 'item' arg is omitted, and java side of this is modified to just pass in 'name', then
    //     // env.get_string(name) works.
    //     // otherwise, here's the error i see:
    //     // get_string result: Some("Couldn\'t get item name: Error(NullPtr(\"get_string obj argument\"), State { next_error: None })")
    //     // Java side looks like this: itemSetName(long itemPtr, String name)

    //     let item = &mut*item;
    //     log(&format!("Got item: {:?}", item)[..]);
        
    //     let result = panic::catch_unwind(|| {
    //         let new_name_res = env.get_string(name).expect("Couldn't get item name");
    //         let new_name: String = String::from(new_name_res);
    //         log(&format!("Got name: {:?}", new_name)[..]);
    //     });

    //     if result.is_ok() {
    //         log("Processed 'name'!");
    //     } else {
    //         log(
    //             &format!(
    //                 "Error updating name: {:?}", result.unwrap_err().downcast_ref::<String>()
    //             )[..]
    //         );
    //         item.name = String::from("Test name from Rust");
    //     }
    // }

    // #[no_mangle]
    // pub unsafe extern fn Java_com_mozilla_toodle_Item_itemSetDueDate(env: JNIEnv, _: JClass, item: *mut Item, due_date: jlong) {
    //     let item = &mut*item;
    //     log(&format!("Got item: {:?}", item)[..]);
    //     log(&format!("Got due date: {:?}", due_date)[..]);

    //     item.due_date = Some(Timespec::new(due_date, 0));
    // }
}