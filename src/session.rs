use crate::{Edition, PlannedTest};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunState {
    Idle,
    Preparing,
    Ready,
    Armed,
    Running,
    Finalizing,
    Complete,
    Aborted,
}

#[derive(Debug, Clone)]
pub struct TestSession {
    pub edition: Edition,
    pub test: PlannedTest,
    pub state: RunState,
    pub output_enabled: bool,
}

impl TestSession {
    pub fn new(edition: Edition, test: PlannedTest) -> Self {
        Self {
            edition,
            test,
            state: RunState::Idle,
            output_enabled: false,
        }
    }

    pub fn prepare(&mut self) -> Result<(), &'static str> {
        if self.state != RunState::Idle {
            return Err("session is not idle");
        }

        self.state = RunState::Preparing;
        self.state = RunState::Ready;
        Ok(())
    }

    pub fn arm(&mut self) -> Result<(), &'static str> {
        if self.state != RunState::Ready {
            return Err("session is not ready");
        }

        if !self.test.intrusive {
            return Err("passive tests do not require arming");
        }

        if self.edition != Edition::Engineer {
            return Err("active tests require Engineer edition");
        }

        self.state = RunState::Armed;
        Ok(())
    }

    pub fn start(&mut self) -> Result<(), &'static str> {
        if self.test.intrusive {
            if self.edition != Edition::Engineer {
                self.output_enabled = false;
                return Err("active tests require Engineer edition");
            }
            if self.state != RunState::Armed {
                self.output_enabled = false;
                return Err("active test must be armed");
            }
            self.output_enabled = true;
        } else if self.state != RunState::Ready {
            return Err("passive test is not ready");
        }

        self.state = RunState::Running;
        Ok(())
    }

    pub fn finish(&mut self) -> Result<(), &'static str> {
        if self.state != RunState::Running {
            return Err("session is not running");
        }

        self.state = RunState::Finalizing;
        self.output_enabled = false;
        self.state = RunState::Complete;
        Ok(())
    }

    pub fn abort(&mut self) {
        self.output_enabled = false;
        self.state = RunState::Aborted;
    }
}
