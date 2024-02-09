pub struct TokenAssessment {
    pub confidence_score: f64, // Score between 0.0 to 1.0, where 1.0 is highest confidence
    pub recommended_trade_amount: f64, // Suggested percentage of the bot's wallet to use for trade
}