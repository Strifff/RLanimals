use tch::{nn, nn::Module, nn::OptimizerConfig, nn::VarStore, Tensor, Kind};

use crate::{NN_RAYS, NN_RAY_DR, NN_RAY_LEN, N_TYPES};

pub struct ActorCritic {
    actor: nn::Sequential,
    critic: nn::Sequential,
}

impl ActorCritic {
    pub fn new(vs: &VarStore, input_size: i64, num_actions: i64) -> ActorCritic {
        let hidden_size: i64 = 512; //todo temp

        let actor = nn::seq()
            .add(nn::linear(vs.root(), input_size, hidden_size, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), hidden_size, num_actions, Default::default()))
            .add_fn(|xs| xs.softmax(-1,Kind::Float));

        let critic = nn::seq()
            .add(nn::linear(vs.root(), input_size, hidden_size, Default::default()))
            .add_fn(|xs| xs.relu())
            .add(nn::linear(vs.root(), hidden_size, 1, Default::default()));

        ActorCritic { actor, critic }
    }

    pub fn forward(&self, t1: &Tensor) -> (Tensor, Tensor) {
        let actor_output = self.actor.forward(&t1);
        let critic_output = self.critic.forward(&t1);
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
        //let critic_loss = advantages.pow(&Tensor::of_slice(&[2])).mean(Kind::Float);
        let loss: Tensor = actor_loss;// + 0.5 * critic_loss;

        optimizer.zero_grad();
        loss.backward();
        optimizer.step();

        println!("Epoch: {}, Loss: {}", epoch, loss);
    }
}

