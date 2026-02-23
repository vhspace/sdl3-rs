extern crate sdl3;
use std::ptr;

use sdl3::properties::*;
use std::ffi::c_char;

#[derive(Debug, Clone)]
#[allow(dead_code)]
struct TestData<'a> {
    pub hello: &'a str,
    pub goodbye: &'a str,
}

impl<'a> Drop for TestData<'a> {
    fn drop(&mut self) {
        println!("TestData dropped: {self:?}");
    }
}

macro_rules! property {
    ($name:expr, $ty:expr) => {
        PropertyName {
            module: "example",
            name: $name,
            short_name: $name,
            value: $name,
            raw: concat!($name, "\0").as_ptr() as *const c_char,
            ty: $ty,
            doc: None,
            available_since: None,
        }
    };
}

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdl3::init().ok();

    let mut properties = Properties::new().unwrap();

    let bprop = property!("bool", PropertyType::BOOLEAN);
    let fprop = property!("float", PropertyType::FLOAT);
    let nprop = property!("number", PropertyType::NUMBER);
    let sprop = property!("string", PropertyType::STRING);
    properties.set(&bprop, true).ok();
    properties.set(&fprop, 6.9f32).ok();
    properties.set(&nprop, 420i64).ok();
    properties.set(&sprop, "blazeit").ok();

    println!("Property bool: {:?}", properties.get(&bprop, false));
    if let Ok(()) = properties.clear("bool") {
        println!("Cleared bool");
    } else {
        println!("Failed to clear bool");
    }
    if let Ok(()) = properties.clear("bool") {
        println!("Cleared bool");
    } else {
        println!("Failed to clear bool");
    }
    println!("Property bool: {:?}", properties.get(&bprop, false));
    println!("Property float: {:?}", properties.get(&fprop, 3.333));
    println!("Property number: {:?}", properties.get(&nprop, -1));

    println!(
        "Property string: {:?}",
        properties.get_string("string", "bad")
    );

    let test = TestData {
        hello: "hello",
        goodbye: "goodbye",
    };

    let pointer_prop = property!("pointer", PropertyType::POINTER);
    let autopointer_prop = property!("autopointer", PropertyType::POINTER);

    // You can set a pointer by yourself, but you have to clean it up
    properties
        .set(&pointer_prop, Box::into_raw(Box::new(test.clone())))
        .ok();

    // Will get a pointer to the data
    if let Ok(pointer) = properties.get(&pointer_prop, ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("Pointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get pointer");
    }

    // Will get a pointer to the data, then claim ownership of it and clear it from the properties object
    // You must clear a pointer property if you claim it
    if let Ok(pointer) = properties.get(&pointer_prop, ptr::null_mut() as *mut TestData) {
        unsafe {
            properties.clear("pointer").ok();
            let value = Box::from_raw(pointer);
            println!("Pointer from box: {value:?}");
        }
    } else {
        println!("Failed to get pointer");
    }

    // The previous get cleared the property, so this will safely fail
    if let Ok(pointer) = properties.get(&pointer_prop, ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("Pointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get pointer");
    }

    // Alternatively to setting a raw pointer, you can set a box which will be automatically cleaned up
    properties
        .set(&autopointer_prop, Box::new(test.clone()))
        .ok();
    // Will get a pointer to the data
    if let Ok(pointer) = properties.get(&autopointer_prop, ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("autopointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get autopointer");
    }
    // The box will be reclaimed and dropped at this point
    properties.clear("autopointer").ok();
    // This will fail
    if let Ok(pointer) = properties.get(&autopointer_prop, ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("autopointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get autopointer");
    }

    // Set the autopointer again
    properties
        .set(&autopointer_prop, Box::new(test.clone()))
        .ok();
    // semi-safely borrow a pointer property by holding a lock on properties
    properties
        .with(&autopointer_prop, |value: &TestData| {
            println!("Borrowed value: {value:?}");
        })
        .ok();
    // Overwrite the property, this will drop the previous value
    properties
        .set(&autopointer_prop, Box::new(test.clone()))
        .ok();

    properties
        .enumerate(Box::new(|properties, name| match name {
            Ok(name) => {
                print!("Enumeration: {name} ");
                if let Ok(thistype) = properties.get_type(name) {
                    match thistype {
                        PropertyType::BOOLEAN => println!("boolean"),
                        PropertyType::FLOAT => println!("float"),
                        PropertyType::NUMBER => println!("number"),
                        PropertyType::STRING => println!("string"),
                        PropertyType::POINTER => println!("pointer"),
                        _ => println!("invalid"),
                    }
                }
            }
            Err(error) => println!("Enumeration error: {error:?}"),
        }))
        .ok();

    // Global properties are not destroyed
    let persistent_prop = property!("persistent", PropertyType::STRING);
    let global = Properties::global().unwrap();
    global.set(&persistent_prop, "rawr x3").ok();
    drop(global);

    let global = Properties::global().unwrap();
    if let Ok(per) = global.get_string("persistent", "3x rwar") {
        println!("Got {per} even after dropping global");
    }

    // Custom cleanup example (called after properties is dropped)
    let test2 = TestData {
        hello: "yes",
        goodbye: "no",
    };
    properties
        .set_with_cleanup(
            "custom_cleanup",
            Box::into_raw(Box::new(test2)),
            Box::new(|data| {
                let test = unsafe { Box::from_raw(data) };
                println!("Custom cleanup of {test:?}");
            }),
        )
        .ok();

    Ok(())
}
