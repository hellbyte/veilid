use super::*;
use crate::crypto::tests_crypto::*;

// Mocks used by various tests

#[must_use]
pub fn fake_latency_stats() -> LatencyStats {
    LatencyStats {
        fastest: TimestampDuration::from(1234),
        average: TimestampDuration::from(2345),
        slowest: TimestampDuration::from(3456),
        tm90: TimestampDuration::from(4567),
        tm75: TimestampDuration::from(5678),
        p90: TimestampDuration::from(6789),
        p75: TimestampDuration::from(7890),
    }
}

#[must_use]
pub fn fake_transfer_stats() -> TransferStats {
    TransferStats {
        total: ByteCount::from(1_000_000),
        maximum: ByteCount::from(3456),
        average: ByteCount::from(2345),
        minimum: ByteCount::from(1234),
    }
}

#[must_use]
pub fn fake_transfer_stats_down_up() -> TransferStatsDownUp {
    TransferStatsDownUp {
        down: fake_transfer_stats(),
        up: fake_transfer_stats(),
    }
}

#[must_use]
pub fn fake_answer_stats() -> AnswerStats {
    AnswerStats {
        span: TimestampDuration::new_secs(10),
        questions: 10,
        answers: 8,
        lost_answers: 0,
        consecutive_answers_maximum: 1,
        consecutive_answers_average: 2,
        consecutive_answers_minimum: 3,
        consecutive_lost_answers_maximum: 4,
        consecutive_lost_answers_average: 5,
        consecutive_lost_answers_minimum: 6,
    }
}

#[must_use]
pub fn fake_rpc_stats() -> RPCStats {
    RPCStats {
        messages_sent: 1_000_000,
        messages_rcvd: 2_000_000,
        questions_in_flight: 42,
        last_question_ts: Some(Timestamp::from(1685569084280)),
        last_seen_ts: Some(Timestamp::from(1685569101256)),
        first_consecutive_seen_ts: Some(Timestamp::from(1685569111851)),
        recent_lost_answers_unordered: 5,
        recent_lost_answers_ordered: 6,
        failed_to_send: 3,
        answer_unordered: fake_answer_stats(),
        answer_ordered: fake_answer_stats(),
    }
}

#[must_use]
pub fn fake_state_stats() -> StateStats {
    StateStats {
        span: TimestampDuration::new_secs(10),
        reliable: TimestampDuration::new_secs(5),
        unreliable: TimestampDuration::new_secs(5),
        dead: TimestampDuration::new_secs(0),
        punished: TimestampDuration::new_secs(0),
        reason: StateReasonStats {
            can_not_send: TimestampDuration::new_secs(1),
            too_many_lost_answers: TimestampDuration::new_secs(2),
            no_ping_response: TimestampDuration::new_secs(3),
            failed_to_send: TimestampDuration::new_secs(4),
            lost_answers: TimestampDuration::new_secs(5),
            not_seen_consecutively: TimestampDuration::new_secs(6),
            in_unreliable_ping_span: TimestampDuration::new_secs(7),
        },
    }
}

#[must_use]
pub fn fake_peer_stats() -> PeerStats {
    PeerStats {
        time_added: Timestamp::from(1685569176894),
        rpc_stats: fake_rpc_stats(),
        latency: Some(fake_latency_stats()),
        transfer: fake_transfer_stats_down_up(),
        state: fake_state_stats(),
    }
}

pub fn fake_peer_table_data() -> PeerTableData {
    PeerTableData {
        node_ids: vec![fake_node_id()],
        peer_address: "123 Main St.".to_string(),
        peer_stats: fake_peer_stats(),
    }
}

pub fn fake_veilid_value_change() -> VeilidValueChange {
    VeilidValueChange {
        key: fake_record_key(),
        subkeys: ValueSubkeyRangeSet::new(),
        count: 5,
        value: Some(
            ValueData::new_with_seq(23.into(), b"ValueData".to_vec(), fake_public_key()).unwrap(),
        ),
    }
}
