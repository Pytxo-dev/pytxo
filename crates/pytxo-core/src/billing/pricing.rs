use crate::billing::{ModelId, TokenCounts};

/// Public list rates in micro-USD per 1M input tokens (Ultra billing basis).
#[derive(Clone, Debug, Default)]
pub struct StaticPriceTable;

impl StaticPriceTable {
    pub fn input_micro_usd_per_million(&self, model: &ModelId) -> i64 {
        match model.provider_hint() {
            Some("anthropic") => 15_000_000,
            Some("openai") => 10_000_000,
            Some("google") => 3_500_000,
            Some("deepseek") => 140_000,
            _ => 10_000_000,
        }
    }

    pub fn output_micro_usd_per_million(&self, model: &ModelId) -> i64 {
        match model.provider_hint() {
            Some("anthropic") => 75_000_000,
            Some("openai") => 30_000_000,
            Some("google") => 10_500_000,
            Some("deepseek") => 280_000,
            _ => 30_000_000,
        }
    }

    pub fn cost_micro_usd(&self, model: &ModelId, counts: &TokenCounts) -> i64 {
        let in_rate = self.input_micro_usd_per_million(model);
        let out_rate = self.output_micro_usd_per_million(model);
        let in_cost = (counts.tokens_in as i128 * in_rate as i128) / 1_000_000;
        let out_cost = (counts.tokens_out as i128 * out_rate as i128) / 1_000_000;
        (in_cost + out_cost) as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::billing::TokenCounts;

    #[test]
    fn deepseek_chat_cost_uses_list_rates() {
        let table = StaticPriceTable;
        let model = ModelId::new("deepseek-chat");
        // 1M in @ 140k micro + 1M out @ 280k micro = 420k micro-USD
        let cost = table.cost_micro_usd(
            &model,
            &TokenCounts {
                tokens_in: 1_000_000,
                tokens_out: 1_000_000,
            },
        );
        assert_eq!(cost, 420_000);
    }
}
