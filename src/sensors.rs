use lm_sensors::{LMSensors, SubFeatureRef};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Matcher {
    pub chip_name: String,
    pub feat_name: String,
    pub feat_label: Option<String>,
    pub sub_feat_name: String,
}

pub trait SubFeatureFinder {
    fn find(&self, matcher: &Matcher) -> anyhow::Result<Option<SubFeatureRef>>;
}

impl SubFeatureFinder for LMSensors {
    fn find(&self, matcher: &Matcher) -> anyhow::Result<Option<SubFeatureRef>> {
        for chip in self.chip_iter(None) {
            // Find chip name
            if chip.name()? != matcher.chip_name {
                continue;
            }
            'feat: for feat in chip.feature_iter() {
                // Find feature name
                let Some(Ok(feat_name)) = feat.name() else {
                    continue 'feat;
                };
                if feat_name != matcher.feat_name {
                    continue 'feat;
                }
                // Find optional label
                if let Some(feat_label) = &matcher.feat_label {
                    if &feat.label()? != feat_label {
                        continue 'feat;
                    }
                }
                'sub_feat: for sub_feat in feat.sub_feature_iter() {
                    // Find sub-feature name
                    let Some(Ok(sub_feat_name)) = sub_feat.name() else {
                        continue 'sub_feat;
                    };
                    if sub_feat_name != matcher.sub_feat_name {
                        continue 'sub_feat;
                    }
                    // Found
                    return Ok(Some(sub_feat));
                }
            }
        }
        // Found nothing
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn test_matcher() {
        let sensors = lm_sensors::Initializer::default().initialize().unwrap();
        let matcher = Matcher {
            chip_name: "coretemp-isa-0000".to_string(),
            feat_name: "temp1".to_string(),
            feat_label: "Package id 0".to_string().into(),
            sub_feat_name: "temp1_input".to_string(),
        };
        let sub_feat = sensors.find(&matcher).unwrap().unwrap();
        dbg!(sub_feat);
        dbg!(sub_feat.value().unwrap().raw_value());
    }
}
