#[macro_export]
macro_rules! impl_crypto_typed_group {
    ($visibility:vis $name:ident) => {
        paste::paste! {

            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), derive(wasm_bindgen_derive::TryFromJsValue))]
            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
            #[derive(Clone, Debug, Serialize, Deserialize, PartialOrd, Ord, PartialEq, Eq, Hash, Default, GetSize)]
            #[serde(from = "Vec<_>", into = "Vec<_>")]
            pub struct [<$name Group>]
            {
                items: Vec<$name>,
            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            make_wasm_bindgen_stubs!([<$name Group>]);

            impl [<$name Group>]
            {
                #[must_use]
                pub fn new() -> Self {
                    Self { items: Vec::new() }
                }
                #[must_use]
                pub fn with_capacity(cap: usize) -> Self {
                    Self {
                        items: Vec::with_capacity(cap),
                    }
                }

                pub fn iter(&self) -> core::slice::Iter<'_, $name> {
                    self.items.iter()
                }

                pub fn add_all_from_slice(&mut self, typed_keys: &[$name]) {
                    'outer: for typed_key in typed_keys {
                        for x in &mut self.items {
                            if x.kind() == typed_key.kind() {
                                *x = typed_key.clone();
                                continue 'outer;
                            }
                        }
                        self.items.push(typed_key.clone());
                    }
                    self.items.sort()
                }

                pub fn contains_any_from_slice(&self, typed_keys: &[$name]) -> bool {
                    typed_keys.iter().any(|typed_key| self.items.contains(typed_key))
                }

            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            #[wasm_bindgen]
            impl [<$name Group>]
            {
                #[must_use]
                pub fn get(&self,
                    #[wasm_bindgen(unchecked_param_type = "CryptoKind")]
                    kind: CryptoKind) -> Option<$name> {
                    self.items.iter().find(|x| x.kind() == kind).cloned()
                }

                pub fn remove(&mut self,
                    #[wasm_bindgen(unchecked_param_type = "CryptoKind")]
                    kind: CryptoKind) -> Option<$name> {
                    if let Some(idx) = self.items.iter().position(|x| x.kind() == kind) {
                        return Some(self.items.remove(idx));
                    }
                    None
                }

                #[wasm_bindgen(js_name = "removeAll")]
                pub fn remove_all(&mut self,
                    #[wasm_bindgen(unchecked_param_type = "CryptoKind[]")]
                    kinds: Vec<CryptoKind>) {
                    for k in kinds {
                        self.remove(k);
                    }
                }
            }
            #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
            impl [<$name Group>]
            {
                #[must_use]
                pub fn get(&self, kind: CryptoKind) -> Option<$name> {
                    self.items.iter().find(|x| x.kind() == kind).cloned()
                }

                pub fn remove(&mut self, kind: CryptoKind) -> Option<$name> {
                    if let Some(idx) = self.items.iter().position(|x| x.kind() == kind) {
                        return Some(self.items.remove(idx));
                    }
                    None
                }

                pub fn remove_all(&mut self, kinds: Vec<CryptoKind>) {
                    for k in kinds {
                        self.remove(k);
                    }
                }
            }


            #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen)]
            impl [<$name Group>] {
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, unchecked_return_type = "CryptoKind[]"))]
                pub fn kinds(&self) -> Vec<CryptoKind> {
                    let mut out = self.items.iter().map(|tk| tk.kind()).collect::<Vec<_>>();
                    out.sort_by(compare_crypto_kind);
                    out
                }

                #[must_use]
                pub fn keys(&self) -> Vec<[<Bare $name>]> {
                    self.items.iter().map(|tk| tk.value()).collect()
                }
                #[must_use]
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = "isEmpty"))]
                pub fn is_empty(&self) -> bool {
                    self.items.is_empty()
                }
                #[must_use]
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(getter, js_name = length))]
                pub fn len(&self) -> usize {
                    self.items.len()
                }
                pub fn contains(&self, typed_key: &$name) -> bool {
                    self.items.contains(typed_key)
                }

                #[must_use]
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = containsAny))]
                pub fn contains_any(&self, typed_keys: Vec<$name>) -> bool {
                    self.contains_any_from_slice(&typed_keys)
                }

                #[must_use]
                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = toArray))]
                pub fn to_vec(&self) -> Vec<$name> {
                    self.items.clone()
                }

                pub fn add(&mut self, typed_key: $name) {
                    for x in &mut self.items {
                        if x.kind() == typed_key.kind() {
                            *x = typed_key;
                            return;
                        }
                    }
                    self.items.push(typed_key);
                    self.items.sort()
                }

                #[cfg_attr(all(target_arch = "wasm32", target_os = "unknown"), wasm_bindgen(js_name = "addAll"))]
                pub fn add_all(&mut self, typed_keys: Vec<$name>) {
                    self.add_all_from_slice(&typed_keys)
                }

                pub fn clear(&mut self) {
                    self.items.clear();
                }


            }

            impl core::ops::Deref for [<$name Group>]
            {
                type Target = [$name];

                #[inline]
                fn deref(&self) -> &[$name] {
                    &self.items
                }
            }

            impl fmt::Display for [<$name Group>]
            {
                fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
                    write!(f, "[")?;
                    let mut first = true;
                    for x in &self.items {
                        if first {
                            first = false;
                        } else {
                            write!(f, ",")?;
                        }
                        write!(f, "{}", x)?;
                    }
                    write!(f, "]")
                }
            }
            impl FromStr for [<$name Group>]
            {
                type Err = VeilidAPIError;
                fn from_str(s: &str) -> Result<Self, Self::Err> {
                    let mut items = Vec::new();
                    if s.len() < 2 {
                        apibail_parse_error!("invalid length", s);
                    }
                    if &s[0..1] != "[" || &s[(s.len() - 1)..] != "]" {
                        apibail_parse_error!("invalid format", s);
                    }
                    for x in s[1..s.len() - 1].split(',') {
                        let tk = $name::from_str(x.trim())?;
                        items.push(tk);
                    }

                    Ok(Self { items })
                }
            }
            impl From<$name> for [<$name Group>]
            {
                fn from(x: $name) -> Self {
                    let mut tks = [<$name Group>]::with_capacity(1);
                    tks.add(x);
                    tks
                }
            }
            impl From<Vec<$name>> for [<$name Group>]
            {
                fn from(x: Vec<$name>) -> Self {
                    let mut tks = [<$name Group>]::with_capacity(x.len());
                    tks.add_all_from_slice(&x);
                    tks
                }
            }
            impl From<&[$name]> for [<$name Group>]
            {
                fn from(x: &[$name]) -> Self {
                    let mut tks = [<$name Group>]::with_capacity(x.len());
                    tks.add_all_from_slice(x);
                    tks
                }
            }
            impl From<[<$name Group>]> for Vec<$name>
            {
                fn from(val: [<$name Group>]) -> Self {
                    val.items
                }
            }

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            #[wasm_bindgen]
            impl [<$name Group>] {
                #[wasm_bindgen(constructor)]
                #[must_use]
                pub fn js_new() -> Self {
                    Self::new()
                }

                #[wasm_bindgen(js_name = parse)]
                pub fn js_parse(s: String) -> VeilidAPIResult<Self> {
                    Self::from_str(&s)
                }

                #[wasm_bindgen(js_name = toString)]
                #[must_use]
                pub fn js_to_string(&self) -> String {
                    self.to_string()
                }

                #[wasm_bindgen(js_name = isEqual)]
                #[must_use]
                pub fn js_is_equal(&self, other: &Self) -> bool {
                    self == other
                }

                // TODO: add more typescript-only operations here
            }
        }
    };
}
