use super::common::Database;

mod embedded {
    use refinery::embed_migrations;
    embed_migrations!("./migrations");
}

pub fn run_migrations(db: &mut Database) {
    match embedded::migrations::runner().run(&mut db.conn) {
        Ok(_) => println!("Migrations ran successfully"),
        Err(e) => println!("Error running migrations: {:?}", e),
    }
}