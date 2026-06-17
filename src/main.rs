fn main() {
    let path = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "test.prisma".to_string());
    let input = std::fs::read_to_string(&path).unwrap_or_else(|e| {
        eprintln!("error reading {path}: {e}");
        std::process::exit(1);
    });
    match transf_orm_cli::importer::prisma::parse_schema(&input) {
        Ok(schema) => println!("{}", schema.to_canonical_json().unwrap()),
        Err(e) => {
            eprintln!("parse error: {e}");
            std::process::exit(1);
        }
    }
}
