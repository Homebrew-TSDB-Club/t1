// use common::{column::FieldType, schema::Schema};
// use hashbrown::HashMap;

// use super::TypeCheck;

// struct TypeSign {
//     input: Vec<FieldType>,
//     output: Vec<FieldType>,
// }

// pub struct Rate {
//     sign: TypeSign,
// }

// impl Rate {
//     unsafe fn call_unchecked(arrays: Vec<FieldType>) {
//         debug_assert_eq!(arrays.len(), 1);
//     }
// }

// pub struct Functions {
//     inner: HashMap<String, TypeSign>,
// }

// impl TypeCheck for Functions {
//     type Output;

//     fn check(self, env: &mut super::Env) -> Result<Self::Output, super::TypeError> {
//         todo!()
//     }
// }
