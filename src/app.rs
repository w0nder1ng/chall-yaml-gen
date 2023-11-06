pub struct GenApp {
    // Example stuff:
    author: String,
    name: String,
    description: String,
    flag_is_file: bool,
    flag: String,
    value: (bool, i32),
    challenge_type: ChallengeType,
    provide: (Vec<String>, i32, bool),
}

#[derive(PartialEq, Eq)]
pub enum ChallengeType {
    TCPBinary,
    WebServer,
    Other,
}
impl Default for GenApp {
    fn default() -> Self {
        Self {
            author: String::new(),
            name: String::new(),
            description: String::new(),
            flag_is_file: false,
            flag: String::new(),
            value: (false, 1),
            challenge_type: ChallengeType::Other,
            provide: (vec![String::new()], 1, false),
        }
    }
}

impl GenApp {
    /// Called once before the first frame.
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        Default::default()
    }

    pub fn to_yaml(&self) -> String {
        let mut yaml = serde_yaml::Mapping::new();
        yaml.insert(
            serde_yaml::Value::String("name".to_string()),
            serde_yaml::Value::String(self.name.clone()),
        );
        yaml.insert(
            serde_yaml::Value::String("author".to_string()),
            serde_yaml::Value::String(self.author.clone()),
        );
        let mut description = self.description.clone();
        if self.challenge_type == ChallengeType::WebServer {
            description.push_str("\n\n{{ link }}");
        } else if self.challenge_type == ChallengeType::TCPBinary {
            description.push_str("\n\n`{{ nc }}`");
        }

        yaml.insert(
            serde_yaml::Value::String("description".to_string()),
            serde_yaml::Value::String(description),
        );
        yaml.insert(
            serde_yaml::Value::String("flag".to_string()),
            match self.flag_is_file {
                false => serde_yaml::Value::String(self.flag.clone()),
                true => {
                    let mut map = serde_yaml::Mapping::new();
                    map.insert(
                        serde_yaml::Value::String("file".to_string()),
                        serde_yaml::Value::String(self.flag.clone()),
                    );
                    serde_yaml::Value::Mapping(map)
                }
            },
        );
        if self.challenge_type == ChallengeType::WebServer {
            if self.provide.2 {
                let yaml_string = "
                - kind: zip
                - spec:
                    as: server.zip
                    files:
                        - server
                    exclude:
                        - server/flag.txt
                    additional:
                        - path: server/flag.txt
                          str: flag{fake_flag}
            ";
                let mut yaml_template =
                    serde_yaml::from_str::<serde_yaml::Value>(yaml_string).unwrap();
                // type YamlString =  serde_yaml::Value::String;

                let mut exclude = Vec::new();
                if self.flag_is_file {
                    use std::ffi::OsStr;
                    use std::path::{Component, Path};
                    let flag_path = Path::new(&self.flag);
                    if flag_path.components().next()
                        != Some(Component::Normal(OsStr::new("server")))
                    {
                        // if the flag would not be in the zip
                        if let serde_yaml::Value::Mapping(ref mut m) = yaml_template[1]["spec"] {
                            m.remove("additional");
                        } else {
                            panic!("fake");
                        }
                    } else {
                        yaml_template[1]["spec"]["additional"][0]["path"] =
                            serde_yaml::Value::String(self.flag.clone());
                        exclude.push(serde_yaml::Value::String(self.flag.clone()));
                    }
                }

                for i in 0..self.provide.1 {
                    if !self.provide.0[i as usize].is_empty() {
                        exclude.push(serde_yaml::Value::String(
                            self.provide.0[i as usize].clone(),
                        ));
                    }
                }
                yaml_template[1]["spec"]["exclude"] = serde_yaml::Value::Sequence(exclude);

                yaml.insert(
                    serde_yaml::Value::String("provide".to_string()),
                    yaml_template,
                );
            }

            let expose = "
            main:
                - target: 5000
                  http: dummy
            ";
            let mut expose = serde_yaml::from_str::<serde_yaml::Value>(expose).unwrap();
            expose["main"][0]["http"] = self.name.clone().into();
            yaml.insert(serde_yaml::Value::String("expose".to_string()), expose);

            let containers = "
            main:
                build: server
                ports:
                    - 5000
            ";
            yaml.insert(
                serde_yaml::Value::String("containers".to_string()),
                serde_yaml::from_str::<serde_yaml::Value>(containers).unwrap(),
            );
        } else {
            if self.challenge_type == ChallengeType::TCPBinary {
                let containers = "
                main:
                    build: bin
                    replicas: 1
                    ports:
                        - 5000
                    k8s:
                        container:
                            securityContext:
                                readOnlyRootFilesystem: true
                                capabilities:
                                    drop:
                                        - all
                                    add:
                                        - chown
                                        - setuid
                                        - setgid
                                        - sys_admin
                        metadata:
                            annotations:
                                container.apparmor.security.beta.kubernetes.io/main: unconfined
                ";
                yaml.insert(
                    serde_yaml::Value::String("containers".to_string()),
                    serde_yaml::from_str::<serde_yaml::Value>(containers).unwrap(),
                );

                let expose = "
                main:
                    - target: 5000
                      tcp: CHANGE_ME
                ";
                yaml.insert(
                    serde_yaml::Value::String("expose".to_string()),
                    serde_yaml::from_str::<serde_yaml::Value>(expose).unwrap(),
                );
            }
            if self.provide.2 {
                let mut seq = Vec::new();
                for i in 0..self.provide.1 {
                    seq.push(serde_yaml::Value::String(
                        self.provide.0[i as usize].clone(),
                    ));
                }
                yaml.insert(
                    serde_yaml::Value::String("provide".to_string()),
                    serde_yaml::Value::Sequence(seq),
                );
            }
        }
        serde_yaml::to_string(&yaml).unwrap()
    }
}

impl eframe::App for GenApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        // Put your widgets into a `SidePanel`, `TopPanel`, `CentralPanel`, `Window` or `Area`.
        // For inspiration and more examples, go to https://emilk.github.io/egui

        egui::CentralPanel::default().show(ctx, |ui| {
            // The central panel the region left after adding TopPanel's and SidePanel's
            ui.heading("challenge.yaml generator");
            // ui.separator();
            ui.horizontal(|ui| {
                ui.label("Author: ");
                ui.text_edit_singleline(&mut self.author);
            });

            ui.horizontal(|ui| {
                ui.label("Name: ");
                ui.text_edit_singleline(&mut self.name);
            });

            ui.horizontal(|ui| {
                ui.label("Description: ");
                ui.text_edit_multiline(&mut self.description);
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.flag_is_file, "flag is file?");
                ui.label("Flag: ");
                ui.text_edit_singleline(&mut self.flag);
            });

            ui.horizontal(|ui| {
                ui.checkbox(&mut self.value.0, "static point value?");
                if self.value.0 {
                    ui.add(
                        egui::DragValue::new(&mut self.value.1)
                            .speed(1.0)
                            .clamp_range(0..=1000),
                    );
                }
            });

            ui.radio_value(
                &mut self.challenge_type,
                ChallengeType::WebServer,
                "website",
            );
            ui.radio_value(
                &mut self.challenge_type,
                ChallengeType::TCPBinary,
                "jailed binary",
            );
            ui.radio_value(&mut self.challenge_type, ChallengeType::Other, "other");

            ui.checkbox(&mut self.provide.2, "provide files?");
            if self.provide.2 {
                if self.challenge_type != ChallengeType::WebServer {
                    ui.label("provide: ");
                } else {
                    ui.label("exclude: ");
                }
                for i in 0..self.provide.1 {
                    ui.text_edit_singleline(&mut self.provide.0[i as usize]);
                }
                ui.horizontal(|ui| {
                    if ui.button("Add").clicked() {
                        self.provide.0.push(String::new());
                        self.provide.1 += 1;
                    }
                    if self.provide.1 >= 1 && ui.button("Remove").clicked() {
                        self.provide.0.pop();
                        self.provide.1 -= 1;
                    }
                });
            }
            if let ChallengeType::TCPBinary = self.challenge_type {
                ui.horizontal(|ui| {
                    ui.label("Remember to change the port to a valid one after generating!");
                });
            }
            if ui.button("copy challenge.yaml").clicked() {
                // self.generated_yaml = ;
                ui.output_mut(|o| o.copied_text = self.to_yaml());
                // let mut file = std::fs::File::create("challenge.yaml").unwrap();
                // file.write_all(yaml.as_bytes()).unwrap();
                // self.has_generated = true;
            }
        });
    }
}
