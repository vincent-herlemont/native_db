extern crate skeptic;

fn main() {
    {
        // Run skeptic
        skeptic::generate_doc_tests(&["README.md"]);
    }
}