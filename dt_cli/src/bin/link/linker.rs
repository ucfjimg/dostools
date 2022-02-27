use dt_lib::error::Error as LinkError;

pub struct Object {
    pub name: String,
    pub image: Vec<u8>,
}

pub struct Library {
    pub name: String,
    pub image: Vec<u8>,
}

pub struct Objects {
    pub objs: Vec<Object>,
    pub libs: Vec<Library>
}

impl Objects {
    pub fn new() -> Objects {
        Objects {
            objs: Vec::new(),
            libs: Vec::new(),
        }
    }

    pub fn add_object(&mut self, name: &str, image: Vec<u8>) {
        self.objs.push(Object{ name: name.to_string(), image });
    }

    pub fn add_library(&mut self, name: &str, image: Vec<u8>) {
        self.libs.push(Library{ name: name.to_string(), image });
    }
}

