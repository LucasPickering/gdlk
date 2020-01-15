//! General utility functions and types.

use lazy_static::lazy_static;
use regex::Regex;
#[cfg(test)]
pub use tests::*;

/// Normalizes the given path, then splits it into segments. Normalization
/// includes de-duplicating slashes and removing trailing slashes.
///
/// In the future, if we want to start supporting relative paths, this could
/// resolve `.` and `..` as well.
///
/// ```
/// assert_eq!(split_path("/").as_slice(), &[""]);
/// assert_eq!(split_path("/dir1").as_slice(), &["", "dir1"]);
/// assert_eq!(split_path("/dir1/dir2/").as_slice(), &["", "dir1", "dir2"]);
/// ```
///
/// See unit tests for more test cases.
pub fn resolve_path(path: &str) -> Vec<&str> {
    lazy_static! {
        static ref PATH_SEP_RGX: Regex = Regex::new(r"/+").unwrap();
    }
    match path {
        // Special case, so that an empty path matches the root
        "" => vec![""],
        _ => PATH_SEP_RGX.split(path).collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use diesel::{Connection, PgConnection};

    /// Helper to create a database connection for testing. This establishes
    /// the connection, then starts a test transaction on it so that no changes
    /// will actually be written to the DB.
    pub fn test_connection() -> PgConnection {
        let database_url = std::env::var("DATABASE_URL").unwrap();
        let conn = PgConnection::establish(&database_url).unwrap();
        conn.begin_test_transaction().unwrap();
        conn
    }

    #[test]
    fn test_resolve_path() {
        assert_eq!(resolve_path("").as_slice(), &[""]);
        assert_eq!(resolve_path("/").as_slice(), &[""]);
        assert_eq!(resolve_path("//").as_slice(), &[""]);
        assert_eq!(resolve_path("dir1").as_slice(), &["dir1"]);
        assert_eq!(resolve_path("/dir1").as_slice(), &["", "dir1"]);
        assert_eq!(
            resolve_path("/dir1/dir2").as_slice(),
            &["", "dir1", "dir2"]
        );
        assert_eq!(
            resolve_path("/dir1///dir2/").as_slice(),
            &["", "dir1", "dir2"]
        );
        assert_eq!(
            resolve_path("/dir1/dir2/file.txt").as_slice(),
            &["", "dir1", "dir2", "file.txt"]
        );
        assert_eq!(
            resolve_path("/dir1/./file.txt").as_slice(),
            &["", "dir1", ".", "file.txt"]
        );
        assert_eq!(
            resolve_path("/dir1/../file.txt").as_slice(),
            &["", "dir1", "..", "file.txt"]
        );
    }
}
