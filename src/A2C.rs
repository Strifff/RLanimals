use tch::{nn, nn::Module, nn::OptimizerConfig, nn::VarStore, Tensor, Kind};
use tch::kind::{FLOAT_CPU, INT64_CPU};
use std::collections::HashMap;
use crate::conc::{Msg, BeastUpdate};
use crate::mpsc::{Sender/*,Receiver*/};

use crate::{NN_RAYS, NN_RAY_DR, NN_RAY_LEN, N_TYPES};

pub struct ActorCritic {
    wall: nn::Sequential,
    plant: nn::Sequential,
    herbi: nn::Sequential,
    carni: nn::Sequential,
    all: nn::Sequential,
    actor: nn::Sequential,
    critic: nn::Sequential,
}

#[derive(Clone)]
pub struct States {
    pub state: ((f64,f64), i32, f64, f64, f64),
    pub memory: Vec<(String, (f64, f64), i32, f64)>,
    pub action: i64,
    pub reward: f64,
    pub state_new: ((f64,f64), i32, f64, f64, f64)
}



impl ActorCritic {
    pub fn new(vs: &VarStore, input_size: i64, num_actions: i64) -> ActorCritic {

        let size_full = (NN_RAYS*NN_RAY_LEN) as i64;
        let size_half = (size_full as f64 * 0.5) as i64;
        //todo change size_all1 to match all input and self-state
        let size_all1 = (size_half as f64 * 2.0 + 0.0) as i64;
        let size_all2 = (size_all1 as f64 * 0.75) as i64;
        let size_all3 = (size_all1 as f64 * 0.5) as i64;

        let size_actor1 = (size_all3 as f64 * 0.75) as i64;
        let size_actor2 = (size_all3 as f64 * 0.5) as i64;

        let size_critic1 = (size_all3 as f64 * 0.75) as i64;
        let size_critic2 = (size_all3 as f64 * 0.5) as i64;

        let wall = nn::seq()
            .add(nn::linear(vs.root(), size_full, size_half,  Default::default()))
            .add_fn(|xs| xs.relu());

        let plant = nn::seq()
            .add(nn::linear(vs.root(), size_full, size_half, Default::default()))
            .add_fn(|xs| xs.relu());

        let herbi = nn::seq()
            .add(nn::linear(vs.root(), size_full, size_half, Default::default()))
            .add_fn(|xs| xs.relu());

        let carni = nn::seq()
            .add(nn::linear(vs.root(), size_full, size_half, Default::default()))
            .add_fn(|xs| xs.relu());

        let all = nn::seq()
            .add(nn::linear(vs.root(), size_all1, size_all2, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), size_all2, size_all3, Default::default()))
            .add_fn(|xs| xs.relu());

        let actor = nn::seq()
            .add(nn::linear(vs.root(), size_all3, size_actor1, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), size_actor1, size_actor2, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), size_actor2, num_actions, Default::default()))
            .add_fn(|xs| xs.softmax(-1,Kind::Float));

        let critic = nn::seq()
            .add(nn::linear(vs.root(), size_all3, size_critic1, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), size_critic1, size_critic2, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), size_critic2, 1, Default::default()));

        ActorCritic { wall, plant, herbi, carni, all, actor, critic }
    }

    pub fn forward(&self, w: &Tensor, p: &Tensor) -> (Tensor, Tensor) {

        let mut wall  = self.wall.forward(&w.flatten(0, 1));
        let mut plant = self.plant.forward(&p.flatten(0,1));

        let mut all = tch::Tensor::cat(&[&wall, &plant], 0);

        all = self.all.forward(&all);
        
        let actor_output = self.actor.forward(&all);
        let critic_output = self.critic.forward(&all);
        (actor_output, critic_output)
    }
}

fn main() {
    let vs = VarStore::new(tch::Device::Cpu);

    let num_actions = 3;
    let input_size = 4;
    let hidden_size = 128;
    let lr = 1e-3;

    let model = ActorCritic::new(&vs, input_size, num_actions);

    let mut optimizer = nn::Adam::default().build(&vs, lr).unwrap();

    for epoch in 0..100 {
        /*
        let mut states = Vec::new();
        let mut actions = Vec::new();
        let mut rewards = Vec::new();

        // Collect experiences
        for _ in 0..100 {
            let state = Tensor::randn(&[input_size], (tch::Kind::Float, tch::Device::Cpu));
            let (action_prob, value) = model.forward(&state);
            let action = action_prob.multinomial(1, true);
            let reward = Tensor::randn(&[], (tch::Kind::Float, tch::Device::Cpu));

            states.push(state);
            actions.push(action);
            rewards.push(reward);
        }

        // Compute advantages and value targets
        let (_, values) = model.forward(&Tensor::cat(&states, 0));
        let values = values.squeeze();
        let advantages = Tensor::cat(&rewards, 0) - values;
        let value_targets = Tensor::cat(&rewards, 0);
        

        // Compute actor and critic losses
        let log_probs = model
            .actor
            .forward(&Tensor::cat(&states, 0))
            .log_softmax(-1,Kind::Float);
        let actor_loss = -(advantages * log_probs.gather(-1, &Tensor::cat(&actions, 0), false)).mean(Kind::Float);
        let critic_loss = advantages.pow(&Tensor::of_slice(&[2])).mean(Kind::Float);
        let loss: Tensor = actor_loss;// + 0.5 * critic_loss;

        optimizer.zero_grad();
        loss.backward();
        optimizer.step();

        println!("Epoch: {}, Loss: {}", epoch, loss);
        */
    }
}

