
/// An update entry to the subscribed data structure.
/// This type is generic over the kind of key used to index or access values in the subscribed data structure.
/// # Note
/// This type makes no guarantees about the values at the keys locations.
/// For example if a thread recieves an [`Update::Addition`] it may have already been deleted before you have aquired the lock to read the value.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Update<K> {
    /// An addition to the subscribed data structure.
    /// The new entry can be found at key K in the subscribed data structure.
    Addition(K),
    /// An deletion of an entry in the subscribed data structure.
    /// K is the key where the entry was located in the subscribed data structure.
    Deletion(K),
    /// An mutation to the subscribed data structure.
    /// The mutated entry can be found at key K in the subscribed data structure.
    Mutation(K),
}

impl<K> Update<K> {
    /// Returns `true` if the update is [`Update::Addition`].
    #[must_use]
    #[inline]
    pub const fn is_addition(&self) -> bool {
        matches!(self, Self::Addition(..))
    }

    /// Returns `true` if the update is [`Update::Deletion`].
    #[must_use]
    #[inline]
    pub const fn is_deletion(&self) -> bool {
        matches!(self, Self::Deletion(..))
    }

    /// Returns `true` if the update is [`Update::Mutation`].
    #[must_use]
    #[inline]
    pub const fn is_mutation(&self) -> bool {
        matches!(self, Self::Mutation(..))
    }
}
