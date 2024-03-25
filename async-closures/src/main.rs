use std::{ collections::HashMap };

// static st: HashMap<String, Box<dyn Fn() -> ()>> = HashMap::new();

static fs: FunctionalStorage = FunctionalStorage::new(
    HashMap::<String, Box<dyn Fn() -> ()>>::from([
        (
            "f1".to_string(),
            Box::new(|| {
                println!("doing 1");
                ()
            }) as Box<dyn Fn() -> ()>,
        ),
    ])
);

struct FunctionalStorage {
    storage: HashMap<String, Box<dyn Fn() -> ()>>,
}

impl FunctionalStorage {
    fn new(st: HashMap<String, Box<dyn Fn() -> ()>>) -> FunctionalStorage {
        FunctionalStorage {
            storage: st,
        }
    }

    fn call(&self, name: &str) -> () {
        let func = self.storage.get(name).unwrap();
        let res = func();
        res
    }
}

fn main() {
    // let mut fs = FunctionalStorage::new();
    // fs.addFunc("add2".to_string(), || {
    //     return 2 + 3;
    // });
    // // fs.addFunc("dp1".to_string(), || { println!("displaying 1") });
    // let x = 2;
    // let y = 3;

    // println!("result: {}", fs.call("add2"));
}
