/// A record that can be proven by a machine.
pub trait MachineRecord: Default + Sized + Send + Sync + Clone {
    /// Appends two records together.
    fn append(&mut self, other: &mut Self);
}
