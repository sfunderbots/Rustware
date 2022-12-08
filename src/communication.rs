pub trait Node {
    fn run_once(&mut self) -> Result<(), ()>;
}
