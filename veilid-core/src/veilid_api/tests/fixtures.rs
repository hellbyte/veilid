use crate::crypto::tests::fixtures::*;
use crate::*;

// Fixtures used by various tests

#[must_use]
pub fn fix_latencystats() -> LatencyStats {
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
pub fn fix_transferstats() -> TransferStats {
    TransferStats {
        total: ByteCount::from(1_000_000),
        maximum: ByteCount::from(3456),
        average: ByteCount::from(2345),
        minimum: ByteCount::from(1234),
    }
}

#[must_use]
pub fn fix_transferstatsdownup() -> TransferStatsDownUp {
    TransferStatsDownUp {
        down: fix_transferstats(),
        up: fix_transferstats(),
    }
}

#[must_use]
pub fn fix_answerstats() -> AnswerStats {
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
pub fn fix_rpcstats() -> RPCStats {
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
        answer_unordered: fix_answerstats(),
        answer_ordered: fix_answerstats(),
    }
}

#[must_use]
pub fn fix_statestats() -> StateStats {
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
pub fn fix_peerstats() -> PeerStats {
    PeerStats {
        time_added: Timestamp::from(1685569176894),
        rpc_stats: fix_rpcstats(),
        latency: Some(fix_latencystats()),
        transfer: fix_transferstatsdownup(),
        state: fix_statestats(),
    }
}

pub fn fix_peertabledata() -> PeerTableData {
    PeerTableData {
        node_ids: vec![fix_fake_node_id()],
        peer_address: "123 Main St.".to_string(),
        peer_stats: fix_peerstats(),
    }
}

pub fn fix_fake_veilid_value_change() -> VeilidValueChange {
    VeilidValueChange {
        key: fix_fake_record_key(),
        subkeys: ValueSubkeyRangeSet::new(),
        count: 5,
        value: Some(
            ValueData::new_with_seq(23.into(), b"ValueData".to_vec(), fix_fake_public_key())
                .unwrap(),
        ),
    }
}
