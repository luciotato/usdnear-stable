//! The same as near_sdk::collections::lookup_map, but keeps also the length of the map
//! A persistent map without iterators. Unlike `near_sdk::collections::UnorderedMap` this map
//! doesn't store keys and values separately in vectors, so it can't iterate over keys. But it
//! makes this map more efficient in the number of reads and writes.
use std::marker::PhantomData;

use near_sdk::env;
use near_sdk::borsh::{self,BorshDeserialize, BorshSerialize};

const ERR_KEY_SERIALIZATION: &[u8] = b"Cannot serialize key with Borsh";
const ERR_VALUE_DESERIALIZATION: &[u8] = b"Cannot deserialize value with Borsh";
const ERR_VALUE_SERIALIZATION: &[u8] = b"Cannot serialize value with Borsh";

/// An non-iterable implementation of a map that stores its content directly on the trie.
#[derive(BorshSerialize, BorshDeserialize)]
pub struct PersistentMap<K, V> {
    key_prefix: Vec<u8>,
    len: u64,
    #[borsh_skip]
    el: PhantomData<(K, V)>,
}

impl<K, V> PersistentMap<K, V> {
    /// Create a new map. Use `key_prefix` as a unique prefix for keys.
    pub fn new(key_prefix: Vec<u8>) -> Self {
        Self { key_prefix, len:0, el: PhantomData }
    }

    pub fn len(&self) -> u64 { self.len }

    fn raw_key_to_storage_key(&self, raw_key: &[u8]) -> Vec<u8> {
        return [&self.key_prefix, raw_key].concat();
    }

    /// Returns `true` if the serialized key is present in the map.
    fn contains_key_raw(&self, key_raw: &[u8]) -> bool {
        let storage_key = self.raw_key_to_storage_key(key_raw);
        env::storage_has_key(&storage_key)
    }

    /// Returns the serialized value corresponding to the serialized key.
    fn get_raw(&self, key_raw: &[u8]) -> Option<Vec<u8>> {
        let storage_key = self.raw_key_to_storage_key(key_raw);
        env::storage_read(&storage_key)
    }

    /// Inserts a serialized key-value pair into the map.
    /// If the map did not have this key present, `None` is returned. Otherwise returns
    /// a serialized value. Note, the keys that have the same hash value are undistinguished by
    /// the implementation.
    pub fn insert_raw(&mut self, key_raw: &[u8], value_raw: &[u8]) -> Option<Vec<u8>> {
        let storage_key = self.raw_key_to_storage_key(key_raw);
        if env::storage_write(&storage_key, value_raw) {
            Some(env::storage_get_evicted().unwrap())
        } else {
            None
        }
    }

    /// Removes a serialized key from the map, returning the serialized value at the key if the key
    /// was previously in the map.
    pub fn remove_raw(&mut self, key_raw: &[u8]) -> Option<Vec<u8>> {
        let storage_key = self.raw_key_to_storage_key(key_raw);
        if env::storage_remove(&storage_key) {
            Some(env::storage_get_evicted().unwrap())
        } else {
            None
        }
    }
}

impl<K, V> PersistentMap<K, V>
where
    K: BorshSerialize,
    V: BorshSerialize + BorshDeserialize,
{
    fn serialize_key(key: &K) -> Vec<u8> {
        match key.try_to_vec() {
            Ok(x) => x,
            Err(_) => env::panic(ERR_KEY_SERIALIZATION),
        }
    }

    fn deserialize_value(raw_value: &[u8]) -> V {
        match V::try_from_slice(&raw_value) {
            Ok(x) => x,
            Err(_) => env::panic(ERR_VALUE_DESERIALIZATION),
        }
    }

    fn serialize_value(value: &V) -> Vec<u8> {
        match value.try_to_vec() {
            Ok(x) => x,
            Err(_) => env::panic(ERR_VALUE_SERIALIZATION),
        }
    }

    /// Returns true if the map contains a given key.
    pub fn contains_key(&self, key: &K) -> bool {
        self.contains_key_raw(&Self::serialize_key(key))
    }

    /// Returns the value corresponding to the key.
    pub fn get(&self, key: &K) -> Option<V> {
        self.get_raw(&Self::serialize_key(key)).map(|value_raw| Self::deserialize_value(&value_raw))
    }

    /// Removes a key from the map, returning the value at the key if the key was previously in the map.
    pub fn remove(&mut self, key: &K) -> Option<V> {
        let value = self.remove_raw(&Self::serialize_key(key));
        match value {
            Some(x) => { 
                //existed previously
                self.len-=1;
                return Some(Self::deserialize_value(&x));
            },
            None => return None
        }
    }

    /// Inserts a key-value pair into the map.
    /// If the map did not have this key present, `None` is returned. Otherwise returns
    /// a value. Note, the keys that have the same hash value are undistinguished by
    /// the implementation.
    pub fn insert(&mut self, key: &K, value: &V) -> Option<V> {
        let value = self.insert_raw(&Self::serialize_key(key), &Self::serialize_value(&value));
        match value {
            Some(x) => { 
                //existed previously
                return Some(Self::deserialize_value(&x))
            },
            None => {
                self.len+=1; //new key
                return None;
            }
        }
    }

    pub fn extend<IT: IntoIterator<Item = (K, V)>>(&mut self, iter: IT) {
        for (el_key, el_value) in iter {
            self.insert(&el_key, &el_value);
        }
    }
}

