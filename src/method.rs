use super::native::JavaMethod;

pub struct MethodId {
    pub native_id: JavaMethod
}

pub struct Method {
    pub id: MethodId
}

pub struct MethodSignature {
    pub name: String,
    pub signature: String,
    pub generic: String
}

impl MethodSignature {

    pub fn new(raw_name:String, raw_signature: String, raw_generic: String) -> MethodSignature {
        MethodSignature { name: raw_name, signature: raw_signature, generic: raw_generic }
    }

    pub fn unknown() -> MethodSignature {
        MethodSignature { name: "<UNKNOWN METHOD>".to_string(), signature: "<UNKNOWN>".to_string(), generic: "<UNKNOWN>".to_string() }
    }
}
