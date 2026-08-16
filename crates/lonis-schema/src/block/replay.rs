// Copyright (C) 2026 Industrial Algebra
// SPDX-License-Identifier: Apache-2.0

//! Replay verification (ADR-0008): checking a block's replay pins against
//! observed hashes.
//!
//! Producers pin hashes in [`ReplayProvenance`] (`input_hash`, `plan_hash`,
//! …) and declare which of them must match in
//! `ReplayMetadata::required_hashes`. `verify_replay` is the consumer
//! side: given the block and the hashes observed at verification time, it
//! reports whether the block may be replayed, and if not, precisely why.

use super::{Block, BlockPayload, ReplayProvenance};

/// The hashes observed at verification time, to check against a block's
/// pinned provenance.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ObservedHashes {
    /// Hash of the inspected project, if observed.
    pub project_hash: Option<String>,
    /// Hash of the explicit input, if observed.
    pub input_hash: Option<String>,
    /// Hash of the plan, if observed.
    pub plan_hash: Option<String>,
    /// Hash of the result, if observed.
    pub result_hash: Option<String>,
}

/// One required hash that did not match.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HashMismatch {
    /// The hash field (`input_hash`, `plan_hash`, …).
    pub field: String,
    /// The hash pinned on the block.
    pub expected: String,
    /// The hash observed now.
    pub observed: String,
}

/// The outcome of replay verification.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReplayStatus {
    /// Every required hash matches (or none are required): the block may be
    /// replayed against these observations.
    Replayable,
    /// The block's provenance declares it non-replayable; `reasons` says why
    /// (e.g. environment-specific runtime state).
    NotReplayable {
        /// The producer's reasons.
        reasons: Vec<String>,
    },
    /// Verification failed: required hashes are missing (absent on the block
    /// or unobserved) or mismatched. Both are reported together.
    Failed {
        /// Required hash fields with no value to compare (absent on the
        /// block, unobserved, or an unknown field name).
        missing: Vec<String>,
        /// Required hashes whose observed values differ.
        mismatches: Vec<HashMismatch>,
    },
}

/// Verify a block's replay pins against the hashes observed now.
///
/// Returns [`ReplayStatus::Replayable`] when `replay.replayable` is set and
/// every field in `required_hashes` is present on both sides and equal.
/// Unknown field names in `required_hashes` count as missing.
#[must_use]
pub fn verify_replay<P: BlockPayload>(block: &Block<P>, observed: &ObservedHashes) -> ReplayStatus {
    let provenance = &block.provenance;
    if !provenance.replay.replayable {
        return ReplayStatus::NotReplayable {
            reasons: provenance.replay.reasons.clone(),
        };
    }

    let mut missing = Vec::new();
    let mut mismatches = Vec::new();
    for field in &provenance.replay.required_hashes {
        let (expected, observed_value) = hash_pair(provenance, observed, field);
        match (expected, observed_value) {
            (Some(expected), Some(observed_value)) if expected != observed_value => {
                mismatches.push(HashMismatch {
                    field: field.clone(),
                    expected: expected.to_owned(),
                    observed: observed_value.to_owned(),
                });
            }
            (Some(_), Some(_)) => {}
            _ => missing.push(field.clone()),
        }
    }

    if missing.is_empty() && mismatches.is_empty() {
        ReplayStatus::Replayable
    } else {
        ReplayStatus::Failed {
            missing,
            mismatches,
        }
    }
}

fn hash_pair<'a>(
    provenance: &'a ReplayProvenance,
    observed: &'a ObservedHashes,
    field: &str,
) -> (Option<&'a str>, Option<&'a str>) {
    match field {
        "project_hash" => (
            provenance.project_hash.as_deref(),
            observed.project_hash.as_deref(),
        ),
        "input_hash" => (
            provenance.input_hash.as_deref(),
            observed.input_hash.as_deref(),
        ),
        "plan_hash" => (
            provenance.plan_hash.as_deref(),
            observed.plan_hash.as_deref(),
        ),
        "result_hash" => (
            provenance.result_hash.as_deref(),
            observed.result_hash.as_deref(),
        ),
        _ => (None, None),
    }
}

#[cfg(test)]
mod tests {
    use super::super::kinds::{BlockKind, Message};
    use super::super::{Attribution, Block, ReplayMetadata, ReplayProvenance};
    use super::*;

    fn attribution() -> Attribution {
        Attribution::new("lonis:test:replay", "lonis:test:replay")
    }

    fn message_block() -> Block<BlockKind> {
        Block::new(
            attribution(),
            BlockKind::Message(Message {
                role: None,
                content: "hello".into(),
                reply_to: None,
            }),
        )
    }

    fn observed() -> ObservedHashes {
        ObservedHashes {
            project_hash: None,
            input_hash: Some("aa".into()),
            plan_hash: Some("bb".into()),
            result_hash: None,
        }
    }

    #[test]
    fn replayable_when_required_hashes_match() {
        let block = message_block().with_provenance(ReplayProvenance {
            input_hash: Some("aa".into()),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["input_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(verify_replay(&block, &observed()), ReplayStatus::Replayable);
    }

    #[test]
    fn replayable_with_no_required_hashes() {
        let block = message_block().with_provenance(ReplayProvenance {
            replay: ReplayMetadata {
                replayable: true,
                ..ReplayMetadata::default()
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(verify_replay(&block, &observed()), ReplayStatus::Replayable);
    }

    #[test]
    fn not_replayable_carries_reasons() {
        let block = message_block().with_provenance(ReplayProvenance {
            replay: ReplayMetadata {
                replayable: false,
                required_hashes: Vec::new(),
                reasons: vec!["runtime capabilities are environment-specific".into()],
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(
            verify_replay(&block, &observed()),
            ReplayStatus::NotReplayable {
                reasons: vec!["runtime capabilities are environment-specific".into()]
            }
        );
    }

    #[test]
    fn mismatched_reports_field_and_values() {
        let block = message_block().with_provenance(ReplayProvenance {
            input_hash: Some("aa".into()),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["input_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        let mut obs = observed();
        obs.input_hash = Some("DIFFERENT".into());
        assert_eq!(
            verify_replay(&block, &obs),
            ReplayStatus::Failed {
                missing: Vec::new(),
                mismatches: vec![HashMismatch {
                    field: "input_hash".into(),
                    expected: "aa".into(),
                    observed: "DIFFERENT".into(),
                }],
            }
        );
    }

    #[test]
    fn missing_when_block_lacks_the_required_hash() {
        let block = message_block().with_provenance(ReplayProvenance {
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["plan_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(
            verify_replay(&block, &observed()),
            ReplayStatus::Failed {
                missing: vec!["plan_hash".into()],
                mismatches: Vec::new(),
            }
        );
    }

    #[test]
    fn missing_when_observation_lacks_the_hash() {
        let block = message_block().with_provenance(ReplayProvenance {
            result_hash: Some("cc".into()),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["result_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(
            verify_replay(&block, &observed()),
            ReplayStatus::Failed {
                missing: vec!["result_hash".into()],
                mismatches: Vec::new(),
            }
        );
    }

    #[test]
    fn unknown_required_field_is_missing() {
        let block = message_block().with_provenance(ReplayProvenance {
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["catalog_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        assert_eq!(
            verify_replay(&block, &observed()),
            ReplayStatus::Failed {
                missing: vec!["catalog_hash".into()],
                mismatches: Vec::new(),
            }
        );
    }

    #[test]
    fn multiple_mismatches_accumulate() {
        let block = message_block().with_provenance(ReplayProvenance {
            input_hash: Some("aa".into()),
            plan_hash: Some("bb".into()),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["input_hash".into(), "plan_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        let mut obs = observed();
        obs.input_hash = Some("x".into());
        obs.plan_hash = Some("y".into());
        let ReplayStatus::Failed { mismatches, .. } = verify_replay(&block, &obs) else {
            panic!("expected mismatches")
        };
        assert_eq!(mismatches.len(), 2);
    }

    #[test]
    fn missing_and_mismatched_report_together() {
        let block = message_block().with_provenance(ReplayProvenance {
            input_hash: Some("aa".into()),
            replay: ReplayMetadata {
                replayable: true,
                required_hashes: vec!["input_hash".into(), "plan_hash".into()],
                reasons: Vec::new(),
            },
            ..ReplayProvenance::default()
        });
        let mut obs = observed();
        obs.input_hash = Some("x".into());
        // plan_hash absent on the block → missing; input_hash wrong → mismatch.
        let status = verify_replay(&block, &obs);
        let ReplayStatus::Failed {
            missing,
            mismatches,
        } = status
        else {
            panic!("expected combined failure, got {status:?}")
        };
        assert_eq!(missing, vec!["plan_hash".to_owned()]);
        assert_eq!(mismatches.len(), 1);
    }
}
