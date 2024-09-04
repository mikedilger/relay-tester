use crate::error::Error;
use crate::results::{set_outcome_by_name, Outcome};
use crate::runner::Runner;
use nostr_types::{Filter, PublicKeyHex};

impl Runner {
    pub async fn test_replaceable_behavior(&mut self) -> Result<(), Error> {
        let (newer_replaceable, ok) = self.event_group_a.get("newer_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = newer_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(newer_replaceable.kind);
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_replaceable_event",
                Outcome::new(false, Some("Replaceable event was not accepted".to_owned())),
            );
        } else {
            self.test_fetch_by_filter_group_a(filter, "find_replaceable_event", Some(1))
                .await;
        }

        /*
        // This should have injected ok, but then been replaced
        let (older_replaceable, ok) = self.event_group_a.get("older_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = older_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(older_replaceable.kind);
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_replaceable_event",
                Outcome::new(false, Some("Replaceable event was not accepted".to_owned()))
            );
        } else {
            self.test_fetch_by_filter_group_a(
                filter,
                "find_replaceable_event"
            ).await;
        }

        (true, "replaceable_event_removes_previous"),
        (true, "replaceable_event_doesnt_remove_future"),
        (true, "parameterized_replaceable_event_removes_previous"),
        (true, "parameterized_replaceable_event_doesnt_remove_future"),

        self.test_fetch_by_filter_group_a(
            filter,
            "find_replaceable_event"
        ).await;

        */

        let (newer_param_replaceable, ok) =
            self.event_group_a.get("newer_param_replaceable").unwrap();
        let filter = {
            let mut filter = Filter::new();
            let pkh: PublicKeyHex = newer_param_replaceable.pubkey.into();
            filter.add_author(&pkh);
            filter.add_event_kind(newer_param_replaceable.kind);
            filter.add_tag_value('d', "1".to_owned());
            filter
        };
        if !ok {
            set_outcome_by_name(
                "find_parameterized_replaceable_event",
                Outcome::new(
                    false,
                    Some("Parameterized replaceable event was not accepted".to_owned()),
                ),
            );
        } else {
            self.test_fetch_by_filter_group_a(
                filter,
                "find_parameterized_replaceable_event",
                Some(1),
            )
            .await;
        }

        /*
        // This should have injected ok, but then been replaced
        let (_older_param_replaceable, _) = self.event_group_a.get("older_param_replaceable").unwrap();
        */

        /*
        "replaceable_event_removes_previous"
        "replaceable_event_doesnt_remove_future"
        "parameterized_replaceable_event_removes_previous"
        "parameterized_replaceable_event_doesnt_remove_future"
         */

        Ok(())
    }

    //        let ephemeral = self.event_group_a.get("ephemeral").unwrap();
    // TBD: Test ephemeral again with a 2nd probe subscribed to see if it shows up when posted
}
