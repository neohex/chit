use hash::H256;
use sha3::Hashable;
use hashdb::{HashDB, DBValue};
use super::{AVLDBMut, AVLMut};

/// A mutable `AVL` implementation which hashes keys and uses a generic `HashDB` backing database.
/// Additionaly it stores inserted hash-key mappings for later retrieval.
///
/// Use it as a `AVL` or `AVLMut` trait object.
pub struct FatDBMut<'db> {
    raw: AVLDBMut<'db>,
}

impl<'db> FatDBMut<'db> {
    /// Create a new avl with the backing database `db` and empty `root`
    /// Initialise to the state entailed by the genesis block.
    /// This guarantees the avl is built correctly.
    pub fn new(db: &'db mut HashDB, root: &'db mut H256) -> Self {
        FatDBMut { raw: AVLDBMut::new(db, root) }
    }

    /// Create a new avl with the backing database `db` and `root`.
    ///
    /// Returns an error if root does not exist.
    pub fn from_existing(db: &'db mut HashDB, root: &'db mut H256) -> super::Result<Self> {
        Ok(FatDBMut { raw: AVLDBMut::from_existing(db, root)? })
    }

    /// Get the backing database.
    pub fn db(&self) -> &HashDB {
        self.raw.db()
    }

    /// Get the backing database.
    pub fn db_mut(&mut self) -> &mut HashDB {
        self.raw.db_mut()
    }

    fn to_aux_key(key: &[u8]) -> H256 {
        key.sha3()
    }
}

impl<'db> AVLMut for FatDBMut<'db> {
    fn root(&mut self) -> &H256 {
        self.raw.root()
    }

    fn is_empty(&self) -> bool {
        self.raw.is_empty()
    }

    fn contains(&self, key: &[u8]) -> super::Result<bool> {
        self.raw.contains(&key.sha3())
    }

    fn get<'a, 'key>(&'a self, key: &'key [u8]) -> super::Result<Option<DBValue>>
        where 'a: 'key
    {
        self.raw.get(&key.sha3())
    }

    fn insert(&mut self, key: &[u8], value: &[u8]) -> super::Result<Option<DBValue>> {
        let hash = key.sha3();
        let out = self.raw.insert(&hash, value)?;
        let db = self.raw.db_mut();

        // don't insert if it doesn't exist.
        if out.is_none() {
            db.emplace(Self::to_aux_key(&hash), DBValue::from_slice(key));
        }
        Ok(out)
    }

    fn remove(&mut self, key: &[u8]) -> super::Result<Option<DBValue>> {
        let hash = key.sha3();
        let out = self.raw.remove(&hash)?;

        // don't remove if it already exists.
        if out.is_some() {
            self.raw.db_mut().remove(&Self::to_aux_key(&hash));
        }

        Ok(out)
    }
}

#[test]
fn fatdb_to_avl() {
    use memorydb::MemoryDB;
    use super::AVLDB;
    use super::AVL;

    let mut memdb = MemoryDB::new();
    let mut root = H256::default();
    {
        let mut t = FatDBMut::new(&mut memdb, &mut root);
        t.insert(&[0x01u8, 0x23], &[0x01u8, 0x23]).unwrap();
    }
    let t = AVLDB::new(&memdb, &root).unwrap();
    assert_eq!(t.get(&(&[0x01u8, 0x23]).sha3()).unwrap().unwrap(),
               DBValue::from_slice(&[0x01u8, 0x23]));
}

