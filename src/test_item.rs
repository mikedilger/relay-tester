use crate::error::Error;
use crate::outcome::Outcome;
use crate::stage::Stage;
use strum_macros::{EnumCount, EnumIter};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, EnumCount, EnumIter)]
#[repr(usize)]
pub enum TestItem {
    // Pre-Auth: nip11
    Nip11Provided,
    ClaimsSupportForNip4,
    ClaimsSupportForNip9,
    ClaimsSupportForNip11,
    ClaimsSupportForNip26,
    ClaimsSupportForNip29,
    ClaimsSupportForNip40,
    ClaimsSupportForNip42,
    ClaimsSupportForNip45,
    ClaimsSupportForNip50,
    ClaimsSupportForNip59,
    ClaimsSupportForNip65,
    ClaimsSupportForNip94,
    ClaimsSupportForNip96,

    // Pre-Auth: auth
    PromptsForAuthInitially,

    // Pre-Auth: eose
    SupportsEose,
    ClosesCompleteSubscriptionsAfterEose,
    KeepsOpenIncompleteSubscriptionsAfterEose,

    // Pre-Auth: public
    PublicCanWrite,
    AcceptsRelayListsFromPublic,
    AcceptsDmRelayListsFromPublic,
    AcceptsEphemeralEventsFromPublic,

    // Registered: reg
    SendsOkAfterEvent,
    VerifiesSignatures,
    VerifiesIdHashes,

    // Registered: json
    AcceptsNip1JsonEscapeSequences,
    AcceptsUnlistedJsonEscapeSequences,
    AcceptsLiteralsForJsonEscapeSequences,
    AcceptsUtf8NonCharacters,

    // Registered: time
    AcceptsEventsOneWeekOld,
    AcceptsEventsOneMonthOld,
    AcceptsEventsOneYearOld,
    AcceptsEventsFromBeforeNostr,
    AcceptsEventsFromBefore2000,
    AcceptsEventsFrom1970,
    AcceptsEventsFromBefore1970,
    AcceptsEventsOneYearIntoTheFuture,
    AcceptsEventsInTheDistantFuture,
    AcceptsEventsWithCreatedAtGreaterThanSigned32Bit,
    AcceptsEventsWithCreatedAtGreaterThanUnsigned32Bit,
    AcceptsEventsWithCreatedAtInScientificNotation,

    // Registered: misc events
    AcceptsEventsWithEmptyTags,

    // Registered: find
    EventsOrderedFromNewestToOldest,
    NewestEventsWhenLimited,
    FindById,
    FindByPubkeyAndKind,
    FindByPubkeyAndTags,
    FindByKindAndTags,
    FindByTags,
    FindByMultipleTags,
    FindByPubkey,
    FindByScrape,
    SinceUntilIncludeEquals,
    LimitZero,

    // Registered: ephemeral
    EphemeralSubscriptionsWork,
    PersistsEphemeralEvents,

    // Registered: replaceables
    AcceptsMetadata,
    ReplacesMetadata,
    AcceptsContactlist,
    ReplacesContactlist,
    ReplacedEventsStillAvailableById,
    ReplaceableEventRemovesPrevious,
    ReplaceableEventDoesntRemoveFuture,
    AddressableEventRemovesPrevious,
    AddressableEventDoesntRemoveFuture,
    FindReplaceableEvent,
    FindAddressableEvent,

    // TBD
    LimitWorksAcrossMultipleFilterGroups,
    ServesPostEoseEvents,
    NoTimeoutWhileSubscribed,
    Nip4DmsRequireAuth,
    /*
    DeleteById,
    RejectsDeleteByIdOfOthers,
    RejectsResubmissionOfDeletedById,
    DeleteByATag,
    NoDeleteByATagOfOthers,
    DeleteByNpAddrPreservesNewer,
    NoResubmissionOfDeletedByNpnaddr,
    NoResubmissionOfOlderThanDeletedByNpnaddr,
    ResubmissionOfNewerThanDeletedByNpnaddr,
    DeleteByPnaddr,
    NoDeleteByPnaddrOfOthers,
    DeleteByPnaddrPreservesNewer,
    DeleteByPnaddrBoundByDtag,
    NoResubmissionOfDeletedByPnaddr,
    NoResubmissionOfOlderThanDeletedByPnaddr,
    ResubmissionOfNewerThanDeletedByPnaddr,
    DeletedReturnsOkFalse,
    */
    CanAuthAsUnknown,
    UnknownCanWriteOwn,
    UnknownCanReadbackOwn,
    UnknownCanWriteOther,
    UnknownCanReadbackOther,
    CanAuthAsKnown,
    KnownCanWriteOwn,
    KnownCanReadbackOwn,
    KnownCanWriteOther,
    KnownCanReadbackOther,
    GiftwrapsRequireAuth,
    LargeContactLists,
    PreservesJsonFieldOrder,
    PreservesNonstandardJsonFields,
    HandlesEventKindLargerThan16bit,
    HandlesFilterKindLargerThan16bit,
    AcceptsNegativeFilterCreatedAt,
    AcceptsNullCharacters,
    HandlesFilterPrefixes,
    MaxSubscriptions,
    MaxConnections,
    AllowsImmediateReconnect,
    IdleTimeoutIfUnsubscribed,
}

impl TestItem {
    pub fn name(&self) -> &'static str {
        use TestItem::*;

        match *self {
            Nip11Provided => "NIP-11 document is provided",
            ClaimsSupportForNip4 => "Claims support for NIP-04 (old DMs)",
            ClaimsSupportForNip9 => "Claims support for NIP-09 (Deletion)",
            ClaimsSupportForNip11 => "Claims support for NIP-11 (Relay Information Document)",
            ClaimsSupportForNip26 => "Claims support for NIP-26 (Delegated Event Signing)",
            ClaimsSupportForNip29 => "Claims support for NIP-29 (Relay-based Groups)",
            ClaimsSupportForNip40 => "Claims support for NIP-40 (Expiration Timestamp)",
            ClaimsSupportForNip42 => "Claims support for NIP-42 (AUTH)",
            ClaimsSupportForNip45 => "Claims supprort for NIP-45 (COUNT)",
            ClaimsSupportForNip50 => "Claims support for NIP-50 (SEARCH)",
            ClaimsSupportForNip59 => "Claims support for NIP-59 (Giftwrap)",
            ClaimsSupportForNip65 => "Claims support for NIP-65 (Relay Lists)",
            ClaimsSupportForNip94 => "Claims support for NIP-94 (File Metadata)",
            ClaimsSupportForNip96 => "Claims support for NIP-96 (HTTP file storage)",
            PromptsForAuthInitially => "Prompts for AUTH when client connects",
            SupportsEose => "Supports EOSE",
            ClosesCompleteSubscriptionsAfterEose => "Closes complete subscriptions after EOSE",
            KeepsOpenIncompleteSubscriptionsAfterEose => {
                "Keeps open incomplete subscriptions after EOSE"
            }
            PublicCanWrite => "Public can write",
            AcceptsRelayListsFromPublic => "Accepts relay lists from the public",
            AcceptsDmRelayListsFromPublic => "Accepts DM relay lists from the public",
            AcceptsEphemeralEventsFromPublic => "Accepts ephemeral events from the public",
            SendsOkAfterEvent => "Sends OK after EVENT",
            VerifiesSignatures => "Verifies event signatures",
            VerifiesIdHashes => "Verifies event ID hashes",
            AcceptsNip1JsonEscapeSequences => "Accepts NIP-01 JSON escape sequences",
            AcceptsUnlistedJsonEscapeSequences => "Accepts unlisted JSON escape sequences",
            AcceptsLiteralsForJsonEscapeSequences => "Accepts literals for JSON escape sequences",
            AcceptsUtf8NonCharacters => "Accepts UTF-8 non-characters",
            AcceptsEventsOneWeekOld => "Accepts event.created_at one week old",
            AcceptsEventsOneMonthOld => "Accepts event.created_at one month old",
            AcceptsEventsOneYearOld => "Accepts event.created_at one year old",
            AcceptsEventsFromBeforeNostr => "Accepts event.created_at from before nostr",
            AcceptsEventsFromBefore2000 => "Accepts event.created_at from before 2000",
            AcceptsEventsFrom1970 => "Accepts event.created_at from 1970",
            AcceptsEventsFromBefore1970 => "Accepts event.created_at from before 1970",
            AcceptsEventsOneYearIntoTheFuture => {
                "Accepts event.created_at one year into the future"
            }
            AcceptsEventsInTheDistantFuture => "Accepts event.created_at in the distant future",
            AcceptsEventsWithCreatedAtGreaterThanSigned32Bit => {
                "Accepts event.created_at greater than signed 32-bit"
            }
            AcceptsEventsWithCreatedAtGreaterThanUnsigned32Bit => {
                "Accepts event.created_at greater than unsigned 32-bit"
            }
            AcceptsEventsWithCreatedAtInScientificNotation => {
                "Accepts event.created_at in scientific notation"
            }
            AcceptsEventsWithEmptyTags => "Accepts events with empty tags",
            EphemeralSubscriptionsWork => "Ephemeral subscriptions work",
            PersistsEphemeralEvents => "Persists ephemeral events",
            EventsOrderedFromNewestToOldest => "Events are ordered from newest to oldest",
            NewestEventsWhenLimited => "Newest events are returned when filter is limited",
            FindById => "Finds by id",
            FindByPubkeyAndKind => "Finds by pubkey and kind",
            FindByPubkeyAndTags => "Finds by pubkey and tags",
            FindByKindAndTags => "Finds by kind and tags",
            FindByTags => "Finds by tags",
            FindByMultipleTags => "Finds by multiple tags",
            FindByPubkey => "Finds by pubkey",
            FindByScrape => "Finds by scrape",
            AcceptsMetadata => "Accepts metadata",
            ReplacesMetadata => "Replaces metadata",
            ReplacedEventsStillAvailableById => "Replaced events are still available by ID",
            AcceptsContactlist => "Accepts Contactlists",
            ReplacesContactlist => "Replaces Contactlists",
            ReplaceableEventRemovesPrevious => "Replaceable events replace older ones",
            ReplaceableEventDoesntRemoveFuture => "Replaceable events don't replace newer ones",
            AddressableEventRemovesPrevious => "Addressable events replace older ones",
            AddressableEventDoesntRemoveFuture => "Addressable events don't replace newer ones",
            FindReplaceableEvent => "Finds replaceable events",
            FindAddressableEvent => "Finds addressable events",
            SinceUntilIncludeEquals => "Since and until filters include endpoints",
            LimitZero => "Limit zero works",
            LimitWorksAcrossMultipleFilterGroups => "Limit works across multiple filter groups",
            ServesPostEoseEvents => "Serves post-EOSE events",
            NoTimeoutWhileSubscribed => "No timeout while subscribed",
            Nip4DmsRequireAuth => "Nip-04 DMs require AUTH",
            GiftwrapsRequireAuth => "Giftwraps require AUTH",
            /*
            DeleteById => "Deletes by id",
            RejectsDeleteByIdOfOthers => "Rejects delete by ID of others",
            RejectsResubmissionOfDeletedById => "Rejects resubmission of deleted by ID",
            DeleteByATag => "Deletes by a-tag",
            NoDeleteByATagOfOthers => "Rejects delete by a-tag of others",
            DeleteByNpAddrPreservesNewer => "DeleteByNpAddrPreservesNewer",
            NoResubmissionOfDeletedByNpnaddr => "ResubmissionOfDeletedByNpnaddr",
            NoResubmissionOfOlderThanDeletedByNpnaddr => "ResubmissionOfOlderThanDeletedByNpnaddr",
            ResubmissionOfNewerThanDeletedByNpnaddr => "ResubmissionOfNewerThanDeletedByNpnaddr",
            DeleteByPnaddr => "DeleteByPnaddr",
            NoDeleteByPnaddrOfOthers => "DeleteByPnaddrOfOthers",
            DeleteByPnaddrPreservesNewer => "DeleteByPnaddrPreservesNewer",
            DeleteByPnaddrBoundByDtag => "DeleteByPnaddrBoundByDtag",
            NoResubmissionOfDeletedByPnaddr => "ResubmissionOfOlderThanDeletedByPnaddr",
            NoResubmissionOfOlderThanDeletedByPnaddr => "ResubmissionOfOlderThanDeletedByPnaddr",
            ResubmissionOfNewerThanDeletedByPnaddr => "ResubmissionOfNewerThanDeletedByPnaddr",
            DeletedReturnsOkFalse => "DeletedReturnsOkFalse",
            */
            CanAuthAsUnknown => "Can AUTH as unknown",
            UnknownCanWriteOwn => "Unknown can write own",
            UnknownCanReadbackOwn => "Unknown can read back own",
            UnknownCanWriteOther => "Unknown can write other",
            UnknownCanReadbackOther => "Unknown can read back other",
            CanAuthAsKnown => "Can AUTH as known",
            KnownCanWriteOwn => "Known can write own",
            KnownCanReadbackOwn => "Known can read back own",
            KnownCanWriteOther => "Known can write other",
            KnownCanReadbackOther => "Known can readback other",
            LargeContactLists => "Supports large contact lists",
            PreservesJsonFieldOrder => "Preserves JSON field order",
            PreservesNonstandardJsonFields => "Preserves Non-standard JSON fields",
            HandlesEventKindLargerThan16bit => "Handles event.kind > 16 bit",
            HandlesFilterKindLargerThan16bit => "Handles filter.kinds > 16 bit",
            AcceptsNegativeFilterCreatedAt => "Accepts negative filter.since/until",
            AcceptsNullCharacters => "Accepts null character",
            HandlesFilterPrefixes => "Handles filter prefixes",
            MaxSubscriptions => "Max subscriptions",
            MaxConnections => "Max connections",
            AllowsImmediateReconnect => "Allows immediate reconnect",
            IdleTimeoutIfUnsubscribed => "Idle timeout if unsubscribed",
        }
    }

    pub fn required(&self) -> bool {
        use TestItem::*;

        match *self {
            Nip11Provided => false,
            ClaimsSupportForNip4 => false,
            ClaimsSupportForNip9 => false,
            ClaimsSupportForNip11 => false,
            ClaimsSupportForNip26 => false,
            ClaimsSupportForNip29 => false,
            ClaimsSupportForNip40 => false,
            ClaimsSupportForNip42 => false,
            ClaimsSupportForNip45 => false,
            ClaimsSupportForNip50 => false,
            ClaimsSupportForNip59 => false,
            ClaimsSupportForNip65 => false,
            ClaimsSupportForNip94 => false,
            ClaimsSupportForNip96 => false,
            PromptsForAuthInitially => false,
            SupportsEose => true,
            ClosesCompleteSubscriptionsAfterEose => false,
            KeepsOpenIncompleteSubscriptionsAfterEose => true,
            PublicCanWrite => false,
            AcceptsRelayListsFromPublic => false,
            AcceptsDmRelayListsFromPublic => false,
            AcceptsEphemeralEventsFromPublic => false,
            SendsOkAfterEvent => true,
            VerifiesSignatures => true,
            VerifiesIdHashes => true,
            AcceptsNip1JsonEscapeSequences => true,
            AcceptsUnlistedJsonEscapeSequences => false,
            AcceptsLiteralsForJsonEscapeSequences => false,
            AcceptsUtf8NonCharacters => true,
            AcceptsEventsOneWeekOld => true,
            AcceptsEventsOneMonthOld => false,
            AcceptsEventsOneYearOld => false,
            AcceptsEventsFromBeforeNostr => false,
            AcceptsEventsFromBefore2000 => false,
            AcceptsEventsFrom1970 => false,
            AcceptsEventsFromBefore1970 => false,
            AcceptsEventsOneYearIntoTheFuture => false,
            AcceptsEventsInTheDistantFuture => false,
            AcceptsEventsWithCreatedAtGreaterThanSigned32Bit => false,
            AcceptsEventsWithCreatedAtGreaterThanUnsigned32Bit => false,
            AcceptsEventsWithCreatedAtInScientificNotation => false,
            AcceptsEventsWithEmptyTags => false,
            EphemeralSubscriptionsWork => false,
            PersistsEphemeralEvents => false,
            EventsOrderedFromNewestToOldest => true,
            NewestEventsWhenLimited => true,
            FindById => true,
            FindByPubkeyAndKind => true,
            FindByPubkeyAndTags => true,
            FindByKindAndTags => true,
            FindByTags => true,
            FindByMultipleTags => true,
            FindByPubkey => true,
            FindByScrape => true,
            AcceptsMetadata => true,
            ReplacesMetadata => true,
            ReplacedEventsStillAvailableById => false,
            AcceptsContactlist => true,
            ReplacesContactlist => true,
            ReplaceableEventRemovesPrevious => true,
            ReplaceableEventDoesntRemoveFuture => true,
            AddressableEventRemovesPrevious => true,
            AddressableEventDoesntRemoveFuture => true,
            FindReplaceableEvent => true,
            FindAddressableEvent => true,
            SinceUntilIncludeEquals => true,
            LimitZero => true,
            LimitWorksAcrossMultipleFilterGroups => true,
            ServesPostEoseEvents => true,
            NoTimeoutWhileSubscribed => true,
            Nip4DmsRequireAuth => false,
            GiftwrapsRequireAuth => true,
            /*
            DeleteById => true,
            RejectsDeleteByIdOfOthers => true,
            RejectsResubmissionOfDeletedById => true,
            DeleteByATag => true,
            NoDeleteByATagOfOthers => true,
            DeleteByNpAddrPreservesNewer => true,
            NoResubmissionOfDeletedByNpnaddr => true,
            NoResubmissionOfOlderThanDeletedByNpnaddr => true,
            ResubmissionOfNewerThanDeletedByNpnaddr => true,
            DeleteByPnaddr => true,
            NoDeleteByPnaddrOfOthers => true,
            DeleteByPnaddrPreservesNewer => true,
            DeleteByPnaddrBoundByDtag => true,
            NoResubmissionOfDeletedByPnaddr => true,
            NoResubmissionOfOlderThanDeletedByPnaddr => true,
            ResubmissionOfNewerThanDeletedByPnaddr => true,
            DeletedReturnsOkFalse => true,
            */
            CanAuthAsUnknown => false,
            UnknownCanWriteOwn => true,
            UnknownCanReadbackOwn => true,
            UnknownCanWriteOther => true,
            UnknownCanReadbackOther => true,
            CanAuthAsKnown => true,
            KnownCanWriteOwn => true,
            KnownCanReadbackOwn => true,
            KnownCanWriteOther => true,
            KnownCanReadbackOther => true,
            LargeContactLists => true,
            PreservesJsonFieldOrder => false,
            PreservesNonstandardJsonFields => false,
            HandlesEventKindLargerThan16bit => false,
            HandlesFilterKindLargerThan16bit => false,
            AcceptsNegativeFilterCreatedAt => false,
            AcceptsNullCharacters => false,
            HandlesFilterPrefixes => false,
            MaxSubscriptions => false,
            MaxConnections => false,
            AllowsImmediateReconnect => false,
            IdleTimeoutIfUnsubscribed => false,
        }
    }

    pub fn stage(&self) -> Stage {
        use TestItem::*;

        match *self {
            // Pre-Auth: nip11
            Nip11Provided => Stage::Preauth,
            ClaimsSupportForNip4 => Stage::Preauth,
            ClaimsSupportForNip9 => Stage::Preauth,
            ClaimsSupportForNip11 => Stage::Preauth,
            ClaimsSupportForNip26 => Stage::Preauth,
            ClaimsSupportForNip29 => Stage::Preauth,
            ClaimsSupportForNip40 => Stage::Preauth,
            ClaimsSupportForNip42 => Stage::Preauth,
            ClaimsSupportForNip45 => Stage::Preauth,
            ClaimsSupportForNip50 => Stage::Preauth,
            ClaimsSupportForNip59 => Stage::Preauth,
            ClaimsSupportForNip65 => Stage::Preauth,
            ClaimsSupportForNip94 => Stage::Preauth,
            ClaimsSupportForNip96 => Stage::Preauth,

            // Pre-Auth: auth
            PromptsForAuthInitially => Stage::Preauth,

            // Pre-Auth: eose
            SupportsEose => Stage::Preauth,
            ClosesCompleteSubscriptionsAfterEose => Stage::Preauth,
            KeepsOpenIncompleteSubscriptionsAfterEose => Stage::Preauth,

            // Pre-Auth: public
            PublicCanWrite => Stage::Preauth,
            AcceptsRelayListsFromPublic => Stage::Preauth,
            AcceptsDmRelayListsFromPublic => Stage::Preauth,
            AcceptsEphemeralEventsFromPublic => Stage::Preauth,

            // Registered: reg
            SendsOkAfterEvent => Stage::Registered,
            VerifiesSignatures => Stage::Registered,
            VerifiesIdHashes => Stage::Registered,

            // Registered: json
            AcceptsNip1JsonEscapeSequences => Stage::Registered,
            AcceptsUnlistedJsonEscapeSequences => Stage::Registered,
            AcceptsLiteralsForJsonEscapeSequences => Stage::Registered,
            AcceptsUtf8NonCharacters => Stage::Registered,

            // Registered: time
            AcceptsEventsOneWeekOld => Stage::Registered,
            AcceptsEventsOneMonthOld => Stage::Registered,
            AcceptsEventsOneYearOld => Stage::Registered,
            AcceptsEventsFromBeforeNostr => Stage::Registered,
            AcceptsEventsFromBefore2000 => Stage::Registered,
            AcceptsEventsFrom1970 => Stage::Registered,
            AcceptsEventsFromBefore1970 => Stage::Registered,
            AcceptsEventsOneYearIntoTheFuture => Stage::Registered,
            AcceptsEventsInTheDistantFuture => Stage::Registered,
            AcceptsEventsWithCreatedAtGreaterThanSigned32Bit => Stage::Registered,
            AcceptsEventsWithCreatedAtGreaterThanUnsigned32Bit => Stage::Registered,
            AcceptsEventsWithCreatedAtInScientificNotation => Stage::Registered,

            // Registered: misc events
            AcceptsEventsWithEmptyTags => Stage::Registered,

            // Registered: find
            EventsOrderedFromNewestToOldest => Stage::Registered,
            NewestEventsWhenLimited => Stage::Registered,
            FindById => Stage::Registered,
            FindByPubkeyAndKind => Stage::Registered,
            FindByPubkeyAndTags => Stage::Registered,
            FindByKindAndTags => Stage::Registered,
            FindByTags => Stage::Registered,
            FindByMultipleTags => Stage::Registered,
            FindByPubkey => Stage::Registered,
            FindByScrape => Stage::Registered,
            SinceUntilIncludeEquals => Stage::Registered,
            LimitZero => Stage::Registered,

            // Registered: ephemeral
            EphemeralSubscriptionsWork => Stage::Registered,
            PersistsEphemeralEvents => Stage::Registered,

            // Registered: replaceables
            AcceptsMetadata => Stage::Registered,
            ReplacesMetadata => Stage::Registered,
            AcceptsContactlist => Stage::Registered,
            ReplacesContactlist => Stage::Registered,
            ReplacedEventsStillAvailableById => Stage::Registered,
            ReplaceableEventRemovesPrevious => Stage::Registered,
            ReplaceableEventDoesntRemoveFuture => Stage::Registered,
            AddressableEventRemovesPrevious => Stage::Registered,
            AddressableEventDoesntRemoveFuture => Stage::Registered,
            FindReplaceableEvent => Stage::Registered,
            FindAddressableEvent => Stage::Registered,

            // TBD
            LimitWorksAcrossMultipleFilterGroups => Stage::Registered,
            ServesPostEoseEvents => Stage::Registered,
            NoTimeoutWhileSubscribed => Stage::Registered,
            CanAuthAsKnown => Stage::Registered,
            KnownCanWriteOwn => Stage::Registered,
            KnownCanReadbackOwn => Stage::Registered,
            KnownCanWriteOther => Stage::Registered,
            KnownCanReadbackOther => Stage::Registered,
            LargeContactLists => Stage::Registered,
            PreservesJsonFieldOrder => Stage::Registered,
            PreservesNonstandardJsonFields => Stage::Registered,
            HandlesEventKindLargerThan16bit => Stage::Registered,
            HandlesFilterKindLargerThan16bit => Stage::Registered,
            AcceptsNegativeFilterCreatedAt => Stage::Registered,
            AcceptsNullCharacters => Stage::Registered,
            HandlesFilterPrefixes => Stage::Registered,
            MaxSubscriptions => Stage::Registered,
            MaxConnections => Stage::Registered,
            AllowsImmediateReconnect => Stage::Registered,
            IdleTimeoutIfUnsubscribed => Stage::Registered,

            // Stranger
            // ...
            Nip4DmsRequireAuth => Stage::Stranger,
            GiftwrapsRequireAuth => Stage::Stranger,
            /*
            DeleteById => Stage::Unknown,
            RejectsDeleteByIdOfOthers => Stage::Unknown,
            RejectsResubmissionOfDeletedById => Stage::Unknown,
            DeleteByATag => Stage::Unknown,
            NoDeleteByATagOfOthers => Stage::Unknown,
            DeleteByNpAddrPreservesNewer => Stage::Unknown,
            NoResubmissionOfDeletedByNpnaddr => Stage::Unknown,
            NoResubmissionOfOlderThanDeletedByNpnaddr => Stage::Unknown,
            ResubmissionOfNewerThanDeletedByNpnaddr => Stage::Unknown,
            DeleteByPnaddr => Stage::Unknown,
            NoDeleteByPnaddrOfOthers => Stage::Unknown,
            DeleteByPnaddrPreservesNewer => Stage::Unknown,
            DeleteByPnaddrBoundByDtag => Stage::Unknown,
            NoResubmissionOfDeletedByPnaddr => Stage::Unknown,
            NoResubmissionOfOlderThanDeletedByPnaddr => Stage::Unknown,
            ResubmissionOfNewerThanDeletedByPnaddr => Stage::Unknown,
            DeletedReturnsOkFalse => Stage::Unknown,
            */
            CanAuthAsUnknown => Stage::Stranger,
            UnknownCanWriteOwn => Stage::Stranger,
            UnknownCanReadbackOwn => Stage::Stranger,
            UnknownCanWriteOther => Stage::Stranger,
            UnknownCanReadbackOther => Stage::Stranger,
        }
    }

    pub async fn run(&self) -> Outcome {
        use TestItem::*;

        use crate::tests::{
            auth, eose, ephemeral, find, json, misc_events, nip11, public, reg, replaceables, tbd,
            time,
        };

        let result = match *self {
            // Pre-Auth: nip11
            Nip11Provided => nip11::nip11_provided().await,
            ClaimsSupportForNip4 => nip11::claimed_support_for_nip(4).await,
            ClaimsSupportForNip9 => nip11::claimed_support_for_nip(9).await,
            ClaimsSupportForNip11 => nip11::claimed_support_for_nip(11).await,
            ClaimsSupportForNip26 => nip11::claimed_support_for_nip(26).await,
            ClaimsSupportForNip29 => nip11::claimed_support_for_nip(29).await,
            ClaimsSupportForNip40 => nip11::claimed_support_for_nip(40).await,
            ClaimsSupportForNip42 => nip11::claimed_support_for_nip(42).await,
            ClaimsSupportForNip45 => nip11::claimed_support_for_nip(45).await,
            ClaimsSupportForNip50 => nip11::claimed_support_for_nip(50).await,
            ClaimsSupportForNip59 => nip11::claimed_support_for_nip(59).await,
            ClaimsSupportForNip65 => nip11::claimed_support_for_nip(65).await,
            ClaimsSupportForNip94 => nip11::claimed_support_for_nip(94).await,
            ClaimsSupportForNip96 => nip11::claimed_support_for_nip(96).await,

            // Pre-Auth: auth
            PromptsForAuthInitially => auth::prompts_for_auth_initially().await,

            // Pre-Auth: eose
            SupportsEose => eose::supports_eose().await,
            ClosesCompleteSubscriptionsAfterEose => {
                eose::closes_complete_subscriptions_after_eose().await
            }
            KeepsOpenIncompleteSubscriptionsAfterEose => {
                eose::keeps_open_incomplete_subscriptions_after_eose().await
            }

            // Pre-Auth: public
            PublicCanWrite => public::public_can_write().await,
            AcceptsRelayListsFromPublic => public::accepts_relay_lists_from_public().await,
            AcceptsDmRelayListsFromPublic => public::accepts_dm_relay_lists_from_public().await,
            AcceptsEphemeralEventsFromPublic => {
                public::accepts_ephemeral_events_from_public().await
            }

            // Registered: reg
            SendsOkAfterEvent => reg::sends_ok_after_event().await,
            VerifiesSignatures => reg::verifies_signatures().await,
            VerifiesIdHashes => reg::verifies_id_hashes().await,

            // Registered: json
            AcceptsNip1JsonEscapeSequences => json::nip1().await,
            AcceptsUnlistedJsonEscapeSequences => json::unlisted().await,
            AcceptsLiteralsForJsonEscapeSequences => json::literals().await,
            AcceptsUtf8NonCharacters => json::utf8non().await,

            // Registered: time
            AcceptsEventsOneWeekOld => time::one_week_ago().await,
            AcceptsEventsOneMonthOld => time::one_month_ago().await,
            AcceptsEventsOneYearOld => time::one_year_ago().await,
            AcceptsEventsFromBeforeNostr => time::before_nostr().await,
            AcceptsEventsFromBefore2000 => time::before_2000().await,
            AcceptsEventsFrom1970 => time::from_1970().await,
            AcceptsEventsFromBefore1970 => time::before_1970().await,
            AcceptsEventsOneYearIntoTheFuture => time::one_year_hence().await,
            AcceptsEventsInTheDistantFuture => time::distant_future().await,
            AcceptsEventsWithCreatedAtGreaterThanSigned32Bit => {
                time::greater_than_signed_32bit().await
            }
            AcceptsEventsWithCreatedAtGreaterThanUnsigned32Bit => {
                time::greater_than_unsigned_32bit().await
            }
            AcceptsEventsWithCreatedAtInScientificNotation => time::scientific_notation().await,

            // Registered: misc_events
            AcceptsEventsWithEmptyTags => misc_events::empty_tags().await,

            // Registered: find
            EventsOrderedFromNewestToOldest => find::newest_to_oldest().await,
            NewestEventsWhenLimited => find::newest_events_when_limited().await,
            FindById => find::find_by_id().await,
            FindByPubkeyAndKind => find::find_by_pubkey_and_kind().await,
            FindByPubkeyAndTags => find::find_by_pubkey_and_tags().await,
            FindByKindAndTags => find::find_by_kind_and_tags().await,
            FindByTags => find::find_by_tags().await,
            FindByMultipleTags => find::find_by_multiple_tags().await,
            FindByPubkey => find::find_by_pubkey().await,
            FindByScrape => find::find_by_scrape().await,
            SinceUntilIncludeEquals => tbd(),
            LimitZero => tbd(),

            // Registered: ephemeral
            EphemeralSubscriptionsWork => ephemeral::ephemeral_subscriptions_work().await,
            PersistsEphemeralEvents => ephemeral::persists_ephemeral_events().await,

            // Registered: replaceables
            AcceptsMetadata => replaceables::accepts_metadata().await,
            ReplacesMetadata => replaceables::replaces_metadata().await,
            AcceptsContactlist => replaceables::accepts_contact_list().await,
            ReplacesContactlist => replaceables::replaces_contact_list().await,
            ReplacedEventsStillAvailableById => {
                replaceables::replaced_events_still_available_by_id().await
            }
            ReplaceableEventRemovesPrevious => tbd(),
            ReplaceableEventDoesntRemoveFuture => tbd(),
            AddressableEventRemovesPrevious => tbd(),
            AddressableEventDoesntRemoveFuture => tbd(),
            FindReplaceableEvent => replaceables::find_replaceable_event().await,
            FindAddressableEvent => replaceables::find_addressable_event().await,

            // TBD
            // EVENTs
            // REQs
            // DELETEs
            /*
            DeleteById => tbd(),
            RejectsDeleteByIdOfOthers => tbd(),
            RejectsResubmissionOfDeletedById => tbd(),
            DeleteByATag => tbd(),
            NoDeleteByATagOfOthers => tbd(),
            DeleteByNpAddrPreservesNewer => tbd(),
            NoResubmissionOfDeletedByNpnaddr => tbd(),
            NoResubmissionOfOlderThanDeletedByNpnaddr => tbd(),
            ResubmissionOfNewerThanDeletedByNpnaddr => tbd(),
            DeleteByPnaddr => tbd(),
            NoDeleteByPnaddrOfOthers => tbd(),
            DeleteByPnaddrPreservesNewer => tbd(),
            DeleteByPnaddrBoundByDtag => tbd(),
            NoResubmissionOfDeletedByPnaddr => tbd(),
            NoResubmissionOfOlderThanDeletedByPnaddr => tbd(),
            ResubmissionOfNewerThanDeletedByPnaddr => tbd(),
            DeletedReturnsOkFalse => tbd(),
            */
            // COMBOs
            LimitWorksAcrossMultipleFilterGroups => tbd(),
            ServesPostEoseEvents => tbd(),
            NoTimeoutWhileSubscribed => tbd(),
            Nip4DmsRequireAuth => tbd(),
            GiftwrapsRequireAuth => tbd(),
            CanAuthAsUnknown => tbd(),
            UnknownCanWriteOwn => tbd(),
            UnknownCanReadbackOwn => tbd(),
            UnknownCanWriteOther => tbd(),
            UnknownCanReadbackOther => tbd(),
            CanAuthAsKnown => tbd(),
            KnownCanWriteOwn => tbd(),
            KnownCanReadbackOwn => tbd(),
            KnownCanWriteOther => tbd(),
            KnownCanReadbackOther => tbd(),
            LargeContactLists => tbd(),
            PreservesJsonFieldOrder => tbd(),
            PreservesNonstandardJsonFields => tbd(),
            HandlesEventKindLargerThan16bit => tbd(),
            HandlesFilterKindLargerThan16bit => tbd(),
            AcceptsNegativeFilterCreatedAt => tbd(),
            AcceptsNullCharacters => tbd(),
            HandlesFilterPrefixes => tbd(),
            MaxSubscriptions => tbd(),
            MaxConnections => tbd(),
            AllowsImmediateReconnect => tbd(),
            IdleTimeoutIfUnsubscribed => tbd(),
        };

        match result {
            Ok(outcome) => outcome,
            Err(e) => match e {
                Error::Disconnected | Error::TimedOut => Outcome::fail(Some(format!("{}", e))),
                other_e => Outcome::err(format!("{}", other_e)),
            },
        }
    }
}
