use super::{
    super::item::{AsDisplay, Slot},
    DynamicObject,
};
use serde::{Deserialize, Serialize};
use std::{
    fmt::Display,
    ops::{Deref, DerefMut},
};

// NOTE: final_goal_square_x/y, load_requires_recalculation are optional and needs serde(default)
// which doesn't work together with serde(flatten), which is needed for DynamicObject.
// Either manually flatten DynamicObject, or remove these fields. For now we go latter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Blockhead {
    #[serde(flatten)]
    obj: DynamicObject,
    pub actions: plist::Value,
    pub clothing_increment_timer: u64,
    pub double_time_unlocked: bool,
    pub interaction_item_index: i64, // could be -1... my god
    pub interaction_item_sub_index: i64,
    pub name: String,
    pub selected_tool_index: u64,
    pub skin_options: plist::Data,
    pub state: plist::Data,
}
inherit!(Blockhead -> DynamicObject, obj);

// An inventory of a blockhead
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Inventory([Slot; Self::NUM_SLOTS]);

impl Inventory {
    pub const NUM_SLOTS: usize = 8;

    pub fn new(slots: [Slot; Self::NUM_SLOTS]) -> Self {
        Self(slots)
    }
}

impl Deref for Inventory {
    type Target = [Slot; Self::NUM_SLOTS];
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Inventory {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Display for Inventory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_list().entries(self.iter().map(AsDisplay)).finish()
    }
}

impl IntoIterator for Inventory {
    type Item = Slot;
    type IntoIter = std::array::IntoIter<Self::Item, { Self::NUM_SLOTS }>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::Inventory;

    #[test]
    fn inventory_round_trip_test() {
        let inventory_data = b"
<?xml version=\"1.0\" encoding=\"UTF-8\"?>
<!DOCTYPE plist PUBLIC \"-//Apple//DTD PLIST 1.0//EN\" \"http://www.apple.com/DTDs/PropertyList-1.0.dtd\">
<plist version=\"1.0\">
<array>
        <array>
                <data>
                AQAAAAAAAAw=
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAgwfiwgAAAAAAAAH7dyxbtNQFAbguX0K4z25sCHkpApNQZUiSNV0
                gM2KrWCRJpZtYfL22KmaFAGpGBiQPi/3v77fOb7rWZxcfL9fR9/yqi62m1H8
                avgyjvLNcpsVm9Uovlu8G7yOL8bnyYvpx8vFp/lVVK6Luonmd29n15dRPAhh
                UpbrPITpYhrNZ9e3i6jrEcLVhziKvzRN+SaEtm2Haa+Gy+19D+swr7ZlXjW7
                Wdds0BUMsyaLu888dP/pOt3brFg24/Oz5Gu+G9dJ6Jdul1ZV2ocn6SzJ0ibd
                p9nqZvLwtKP9STgc/Q5NIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiC
                IAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiC
                IAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAiCIAj6S5SEtKrS3T4e0pPK
                z6vHwvbP7Y/oxB0gCIIgCIIgCIL+X/Ts7NTuq25/nZ2eq3x/8zivnZi6jujE
                7SEIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAIgiAI
                giAIgiAIgiAI+ufo+GOhQ+oOi2XTr+W6qLvwA2esHopJ5wAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAgwfiwgAAAAAAAAH7ZZLk6JIFEbXXb/CcWtUJYKITljVwdMSBAFB
                gR2SKCgvIXn++gZ7pucRHRPR2wlzw5fccy65uUGuvjZxNKr8vAjT5H08fcPG
                Iz/xUhgml/exaQivi/HXj5fVb9yONWyVH2VRWKCRajLbDTsavwJAZ1nkA8AZ
                3EjdbvbGqO8BAK+MR+MAoex3AOq6fnMH6s1L4wEsgJqnmZ+jdts3e+2FN4jg
                uP/M9+7/OE7/FoYe+nj5srr57UexAsOj37l57g7hb+nLCrrIfaSjxtH8he5X
                /f6ogD9LK/DfplP3Ekvrv27ytUZ/X/U5rC9/ZPqTcgL9RBY1L/vnGwYYn17s
                XChcNpGGDR4UMGzKSAd112SLgo1OGwbiLaEgcsKIChuHsCkTQztYOVedKxtC
                yuUHD1QnUSNn2XIWtgIDjQM8r++pJLEyzG5md3DOAelIadIhbjITiF0dA2bw
                YlNqo08+OorUfEHG4fXC1Kx302iBSiWbB1WdHgrNo53Gv6eIrI3r7OGZohsx
                IZJCgvEvWs0LjMJPvNSKZwRPh0Rc7HNW2RZgf4P2rDPCLmEuj3Nim1ykHGap
                Sx06mY3e+ZuFxQiYbC8LZcemta47eMt1FsNZuJdt9kE5ePixtGVNm58vDXkR
                bluNzwJVz21B/pQxe4IK13SUe5DkEJsluYav8dYavClER/dA295ZQFG3nKRx
                oNaZCgNBxrekLVMmZkf1MfTmYCpjpMA0+Hrw7mwYc/BkcOb8wnIct3adPbmX
                7LMFbNEvXJWfVLCM5/Jt6TAl0BNJXQyeVKiLyfFOaEuhadd8t1RwoSTm8VWb
                YF1yXkOTcohkg0RZxQgRNXw8dQcv+hQDv6TQbnPDExP5+G3nddJBcZAx7Q4w
                kQ6xnCQlSGStKVFYqkEbDx45pXZ7xwBuu7VIh6LOXSiibBHnVRH5Z6ut1Paq
                EV45OVWG1xEE6uuDZxPxqdrpJj1LGuNyrWrZykoLcPBwNMRalETd511I6Zqe
                NvrVK8ylnAxeeTXx672+Yt2GWt8agQisjNKJ+zxUTpynzlyO2su8FiKXMqsY
                KW2QTAaPiatMXxZ3vp+oX5sk+8fw/HsGfwrRT+gJPaEn9ISe0P8ZWvwc+ut3
                +iP1xceteQUed+qPl29sHDTy6gsAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/7dlBT4MwFAfw8/gUtXd4ejOmsGxjJkuIw8gO
                89ZBo0QGpDTWfXsLmqkXx8nD/HPh0ffrn97eoWL6tq/Yq9Jd2dQhvwouOVN1
                3hRl/RTyTXbrX/Np5ImLeL3ItumStVXZGZZu5slqwbhPNGvbShHFWczSZPWQ
                MZdBtLzjjD8b094QWWsD2asgb/Y97CjVTau0OSQuzHcbgsIU3P3mI/3Hcdxq
                UeYm8ibiRR2iTlD/cl9Sa9kX36qJKKSRQ7W7n30+Nhw6dGwBAQEBAQH9AyTo
                90m5HhMPBAQEBAR0tujUpNzaEfFAQEBAQEBni05NyscxgxgICAgICOjv0Nfo
                OlauOdwwChruHyPvHUCTe1QWHQAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAAAAAAAAH7d3JjqNWFAbgdfdTON5aXRiXx8hVLcxswGAb
                PO3MjBnNaNfTx1R3p5Moqt5FivSzuQf4/stdcAS7O/96i6NO7eRFkCYvXfKp
                3+04iZXaQeK9dA2d+zLtfn39PP+NUWn9qLGdLAqKsqMZC1mkO90vBEFlWeQQ
                BKMzHU0Wt3rnMQdBsKtup+uXZfY7QTRN83Ru1ZOVxi0sCC1PMycv7/Jjsi+P
                wJNd2t3HY77N/rflPK7agVW+fv40D537azEn2uFxds7zc1v8pfo0t8/l+b3i
                vTX17WjcoPG+15Rw2enmpUwbViCdOpBX6rNAu+ey2MV9qs3R66TcBFI6jlZR
                6AnWyjzlKvnWq8OhxzdXf8pvEqsyldhRb4TtzvwoM9tcKGbNLF5OLnFpNgvJ
                YfPTuHDcW8XEzWGo8sTgMBSKI5GO2EjihHNQmW6by0bausgK0d2oNBXKjMP7
                XH8aFqXXCOur8jyYyId9WtW0Sg7Uxk0k2vXbXHy7pwa9HVFSqBl9bb+iIvUa
                aWbExHfirgkbTt2OJeY2EghKNPxZRBNxmxvf1jO/ZvfsMD1w7NFgA3K5rrf+
                hpmMfD46euubJYwOpnLhN44Ur5Klwb6v05AcZTeIthbVtxYDfee8SRG93ygL
                JY6UU0gRYi/xRV+UyGbbb3aprBFtTlAr8dpcqczSuUmhZ/ZpYnu2WjoscTOv
                Nrnzylnmk5kyXlyCzarM5Xve5qzj826gLupzdEsMf0Tn9YbJfDPlt1src7Le
                ePM2jZcKR7JvsX6KRyGnyW1uMqzdky8H3nCR3+0xeZLE2JQm3vKicdua8EL/
                lLiBYFiziqnL1WySJLM2V5b1dZZcGXrWN8rKdaL8YJJv8vCS8hZDV4VSaMou
                ZenK6q/UZLC3RHbV5taWl0VNzw56zJWcTPhLcp+OS5aMxt6xCquiiarUmpq5
                nVzV4qzXTjXYt7kBwVYXjl8fSFtc3u8cH7OkvN8qhJJ4z5Kgyfltnz337ELV
                7scB75TC4X2d0krvF9k0CiNtnAySft8VblfbDIbr9Ow2HL+/7Ht6Yzxe9ZeX
                944gfrTEnPi4Y5QfPUI1/0j+K6KAgICAgICAgICA/gOE31ggICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAgICAg
                ICAgICAgICAgICAgICAgICAgICAgICCg/yn61Yb12pr9Hvxgw/qf6IM1AAEB
                AQEB/Qr9/Cr9WT1uBlbZjlkUFI/iD75cbuYdlAAA
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/nVBNC8IwDD27X1F736I3kW6jbgrC0InzoLey
                Fh3uo3TF6r+3myh6ETGXvOTlvYSQ8FqV6CJUWzS1j8feCCNR5w0v6qOPd9nC
                neAwcMgwXkfZPp0jWRatRululiwjhF0AKmUpAOIsRmmy3GbIegDMVxjhk9Zy
                CmCM8Vg35eVN1Q22kKpGCqVviTVzrcDjmmO75uH+cY7t8iLXgTMgZ3ELWgJd
                shVTinXgDQ0IZ5r16GCojdmRUuP3DDwpAj8pN/8qqflb+XXnC1myfwiB/l2B
                cwc3T8VwxQEAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAwwfiwgAYA9lXgL/nZBRC4IwFIWf81esveutt4hpmBYEUkL2UG/D
                jZJMxxyt/n1Xo6iXCAdjZzv3O3dcNrtdSnKVuinqyqdjb0SJrPJaFNXRp7ts
                6U7oLHDYMN5E2T5dEFUWjSHpbp6sIkJdgFCpUgLEWUzSZLXNCGYALNaU0JMx
                agpgrfV4W+Xl9aUtbCDVtZLa3BMMcxHwhBEU2zzTv76Dr6LITeAM2Fneg4ZB
                e+CNa81b8aEGTHDDO3WwIa4It/U7B14Wg3/IeX/S9iXD4y/yrdDsBsKgG1fg
                PABFr9BBxQEAAA==
                </data>
        </array>
        <array>
                <data>
                DAAAAAAAAAwfiwgAYA9lXgL/7dlBT4MwFAfw8/gUtXd4ejOmsOCYyRLiSGQH
                d0PaTCKDpjTivr0FzdSL42jmnwuPvl//9PYOFfO3fc1elemqtgn5VXDJmWrK
                VlbNLuSb/M6/5vPIExfJepE/Zkum66qzLNvcpqsF4z5RrHWtiJI8YVm6esiZ
                yyBa3nPGn63VN0R93wfFoIKy3Q+wo8y0Whl7SF2Y7zYE0krufvOR/uM4blVW
                pY28mXhRh6gTNLzcV2FMMRTfqpmQhS3GSvbx59OHY4eOLSAgICAgoH+ABP0+
                KZ92E+KBgICAgIDOFp2alOt4QjwQEBAQENDZolOTcjslHggICAgI6I+hr/l2
                rFxzvIYUNF5SRt47sNC68TsdAAA=
                </data>
        </array>
</array>
</plist>";
        let inventory: Inventory = plist::from_reader_xml(inventory_data.as_slice())
            .expect("should be able to deserialize");
        let mut round_trip_inventory_data = Vec::new();
        plist::to_writer_xml(&mut round_trip_inventory_data, &inventory).expect("should serialize");
        let round_trip_inventory: Inventory =
            plist::from_reader_xml(round_trip_inventory_data.as_slice())
                .expect("should be able to deserialize");
        assert_eq!(inventory, round_trip_inventory);
    }
}
