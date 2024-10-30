pub mod SystemClock{
    use std::time::Duration;
    use tokio::time::sleep;
    pub struct SystemClock<'a> {
        phase: u32,
        phases: u32,
        pub quadrature_clocks: Vec<bool>,
        frequency: u32,
        tick_callback: Box<dyn FnMut() + 'a>,
    }
    
    impl<'a> SystemClock<'a> {
        pub fn new<F>(phases: u32, frequency: u32, tick_callback: F) -> Self 
        where 
            F: FnMut() + 'a,
        {
            Self {
                phase: 0,
                phases,
                quadrature_clocks: vec![false; phases as usize],
                frequency,
                tick_callback: Box::new(tick_callback),
            }
        }
    
        fn tick(&mut self) {
            self.phase += 1;
            for clk in &mut self.quadrature_clocks {
                *clk = false;
            }
            self.quadrature_clocks[(self.phase % self.phases) as usize] = true;
            (self.tick_callback)();
        }
    
        pub async fn start(&mut self) {
            let tick_duration = Duration::from_secs_f64(1.0 / self.frequency as f64);
            // Start the clock
            loop {
                self.tick();
                sleep(tick_duration).await;
            }
        }
    }
}
