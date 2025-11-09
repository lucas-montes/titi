
use std::path::PathBuf;



/// Configuration received from the user specifiying endpoint, schema, auth, etc...
pub struct Configuration{
    schema: PathBuf,
    tests: Vec<Test>
    // TODO: add more fields
}

struct Test {
    endpoint: String,
    // TODO: qdd more fields
}
