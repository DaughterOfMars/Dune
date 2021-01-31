use super::*;

pub(crate) struct PhaseStages {
    update: Box<dyn Stage>,
    enter: Box<dyn Stage>,
    exit: Box<dyn Stage>,
}

impl Default for PhaseStages {
    fn default() -> Self {
        Self {
            enter: Box::new(SystemStage::parallel()),
            update: Box::new(SystemStage::parallel()),
            exit: Box::new(SystemStage::parallel()),
        }
    }
}

pub(crate) struct PhaseStage<T> {
    stages: HashMap<Phase, PhaseStages>,
    states: HashSet<Discriminant<T>>,
}

impl<T> Default for PhaseStage<T> {
    fn default() -> Self {
        PhaseStage {
            stages: Default::default(),
            states: Default::default(),
        }
    }
}

impl<T> PhaseStage<T> {
    pub fn add_valid_state(&mut self, state: T) -> &mut Self {
        self.states.insert(std::mem::discriminant(&state));
        self
    }

    pub fn valid_states(&mut self, states: Vec<T>) -> &mut Self {
        for state in states.iter() {
            self.states.insert(std::mem::discriminant(&state));
        }
        self
    }

    pub fn on_phase_enter<S: System<In = (), Out = ()>>(
        &mut self,
        phase: Phase,
        system: S,
    ) -> &mut Self {
        self.enter_phase(phase, |system_stage: &mut SystemStage| {
            system_stage.add_system(system)
        })
    }

    pub fn on_phase_exit<S: System<In = (), Out = ()>>(
        &mut self,
        phase: Phase,
        system: S,
    ) -> &mut Self {
        self.exit_phase(phase, |system_stage: &mut SystemStage| {
            system_stage.add_system(system)
        })
    }

    pub fn on_phase_update<S: System<In = (), Out = ()>>(
        &mut self,
        phase: Phase,
        system: S,
    ) -> &mut Self {
        self.update_phase(phase, |system_stage: &mut SystemStage| {
            system_stage.add_system(system)
        })
    }

    pub fn enter_phase<S: Stage, F: FnOnce(&mut S) -> &mut S>(
        &mut self,
        phase: Phase,
        func: F,
    ) -> &mut Self {
        let stages = self.phase_stages(phase);
        func(
            stages
                .enter
                .downcast_mut()
                .expect("'Enter' stage does not match the given type"),
        );
        self
    }

    pub fn exit_phase<S: Stage, F: FnOnce(&mut S) -> &mut S>(
        &mut self,
        phase: Phase,
        func: F,
    ) -> &mut Self {
        let stages = self.phase_stages(phase);
        func(
            stages
                .exit
                .downcast_mut()
                .expect("'Exit' stage does not match the given type"),
        );
        self
    }

    pub fn update_phase<S: Stage, F: FnOnce(&mut S) -> &mut S>(
        &mut self,
        phase: Phase,
        func: F,
    ) -> &mut Self {
        let stages = self.phase_stages(phase);
        func(
            stages
                .update
                .downcast_mut()
                .expect("'Update' stage does not match the given type"),
        );
        self
    }

    fn phase_stages(&mut self, phase: Phase) -> &mut PhaseStages {
        self.stages.entry(phase).or_default()
    }
}

impl<T: Resource + Clone> Stage for PhaseStage<T> {
    fn initialize(&mut self, world: &mut World, resources: &mut Resources) {
        for state_stages in self.stages.values_mut() {
            state_stages.enter.initialize(world, resources);
            state_stages.update.initialize(world, resources);
            state_stages.exit.initialize(world, resources);
        }
    }

    fn run(&mut self, world: &mut World, resources: &mut Resources) {
        let state = resources
            .get::<State<T>>()
            .expect("Missing state resource")
            .current()
            .clone();

        if self.states.is_empty() || self.states.contains(&std::mem::discriminant(&state)) {
            loop {
                if let (curr_phase, Some(next_phase)) = {
                    let phase = resources
                        .get::<GamePhase>()
                        .expect("Missing game phase resource");
                    (phase.curr, phase.next.front().cloned())
                } {
                    if let Some(current_phase_stages) = self.stages.get_mut(&curr_phase) {
                        current_phase_stages.exit.run(world, resources);
                    }

                    if let Some(next_phase_stages) = self.stages.get_mut(&next_phase) {
                        next_phase_stages.enter.run(world, resources);
                    }

                    resources
                        .get_mut::<GamePhase>()
                        .expect("Missing game phase resource")
                        .apply();
                } else {
                    break;
                }
            }

            let curr_phase = resources
                .get::<GamePhase>()
                .expect("Missing game phase resource")
                .curr;

            if let Some(current_phase_stages) = self.stages.get_mut(&curr_phase) {
                current_phase_stages.update.run(world, resources);
            }
        }
    }
}
