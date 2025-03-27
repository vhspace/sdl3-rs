extern crate sdl3;
use std::ptr;

use sdl3::properties::*;

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

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    sdl3::init().ok();

    let mut properties = Properties::new().unwrap();

    let bprop = "bool";
    let fprop = "float";
    let nprop = "number";
    let sprop = "string";
    properties.set(bprop, true).ok();
    properties.set(fprop, 6.9).ok();
    properties.set(nprop, 420).ok();
    properties.set(sprop, "blazeit").ok();

    println!("Property {bprop}: {:?}", properties.get(bprop, false));
    if let Ok(()) = properties.clear(bprop) {
        println!("Cleared {bprop}");
    } else {
        println!("Failed to clear {bprop}");
    }
    if let Ok(()) = properties.clear(bprop) {
        println!("Cleared {bprop}");
    } else {
        println!("Failed to clear {bprop}");
    }
    println!("Property {bprop}: {:?}", properties.get(bprop, false));
    println!("Property {fprop}: {:?}", properties.get(fprop, 3.333));
    println!("Property {nprop}: {:?}", properties.get(nprop, -1));
    println!("Property nodefault: {:?}", properties.get("nodefault", -1));
    println!("Property nodefault: {:?}", properties.get("nodefault", 42));

    println!(
        "Property {sprop}: {:?}",
        properties.get_string(sprop, "bad")
    );

    let test = TestData {
        hello: "hello",
        goodbye: "goodbye",
    };

    // You can set a pointer by yourself, but you have to clean it up
    properties
        .set("pointer", Box::into_raw(Box::new(test.clone())))
        .ok();

    // Will get a pointer to the data
    if let Ok(pointer) = properties.get("pointer", ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("Pointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get pointer");
    }

    // Will get a pointer to the data, then claim ownership of it and clear it from the properties object
    // You must clear a pointer property if you claim it
    if let Ok(pointer) = properties.get("pointer", ptr::null_mut() as *mut TestData) {
        unsafe {
            properties.clear("pointer").ok();
            let value = Box::from_raw(pointer);
            println!("Pointer from box: {:?}", value);
        }
    } else {
        println!("Failed to get pointer");
    }

    // The previous get cleared the property, so this will safely fail
    if let Ok(pointer) = properties.get("pointer", ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("Pointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get pointer");
    }

    // Alternatively to setting a raw pointer, you can set a box which will be automatically cleaned up
    properties.set("autopointer", Box::new(test.clone())).ok();
    // Will get a pointer to the data
    if let Ok(pointer) = properties.get("autopointer", ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("autopointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get autopointer");
    }
    // The box will be reclaimed and dropped at this point
    properties.clear("autopointer").ok();
    // This will fail
    if let Ok(pointer) = properties.get("autopointer", ptr::null_mut() as *mut TestData) {
        unsafe {
            println!("autopointer: {:?}", *pointer);
        }
    } else {
        println!("Failed to get autopointer");
    }

    // Set the autopointer again
    properties.set("autopointer", Box::new(test.clone())).ok();
    // semi-safely borrow a pointer property by holding a lock on properties
    properties
        .with("autopointer", |value: &TestData| {
            println!("Borrowed value: {value:?}");
        })
        .ok();
    // Overwrite the property, this will drop the previous value
    properties.set("autopointer", Box::new(test.clone())).ok();

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
    let global = Properties::global().unwrap();
    global.set("persistent", "rawr x3").ok();
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
