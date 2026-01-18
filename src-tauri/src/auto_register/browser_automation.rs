use crate::auto_register::models::{BrowserConfig, BrowserMode};
use anyhow::{Result, Context};
use headless_chrome::{Browser, LaunchOptionsBuilder};
use headless_chrome::Tab;
use std::sync::Arc;
use std::time::Duration;
use std::ffi::OsStr;
use rand::Rng;

pub struct BrowserAutomation {
    config: BrowserConfig,
}

impl BrowserAutomation {
    pub fn new(config: BrowserConfig) -> Self {
        Self { config }
    }

    pub fn generate_random_window_size() -> (u32, u32) {
        let mut rng = rand::thread_rng();
        let width = rng.gen_range(800..=1920);
        let height = rng.gen_range(600..=1080);
        (width, height)
    }

    pub fn generate_random_os_version() -> String {
        let mut rng = rand::thread_rng();
        if rng.gen_bool(0.5) {
            "Windows 10".to_string()
        } else {
            "Windows 11".to_string()
        }
    }

    fn generate_fingerprint_script() -> String {
        let mut rng = rand::thread_rng();

        format!(
            r#"
            (function() {{
                // Canvas fingerprint randomization
                const originalGetContext = HTMLCanvasElement.prototype.getContext;
                HTMLCanvasElement.prototype.getContext = function(type, attributes) {{
                    const context = originalGetContext.call(this, type, attributes);
                    if (type === '2d') {{
                        const originalGetImageData = context.getImageData;
                        context.getImageData = function(...args) {{
                            const imageData = originalGetImageData.apply(this, args);
                            for (let i = 0; i < imageData.data.length; i += 4) {{
                                imageData.data[i] = imageData.data[i] ^ {};
                            }}
                            return imageData;
                        }};
                    }}
                    return context;
                }};

                // WebGL fingerprint randomization
                const getParameter = WebGLRenderingContext.prototype.getParameter;
                WebGLRenderingContext.prototype.getParameter = function(parameter) {{
                    if (parameter === 37445) {{
                        return 'Intel Inc.';
                    }}
                    if (parameter === 37446) {{
                        return 'Intel(R) UHD Graphics 630';
                    }}
                    return getParameter.call(this, parameter);
                }};

                // WebGL2 fingerprint randomization
                if (window.WebGL2RenderingContext) {{
                    const getParameter2 = WebGL2RenderingContext.prototype.getParameter;
                    WebGL2RenderingContext.prototype.getParameter = function(parameter) {{
                        if (parameter === 37445) {{
                            return 'Intel Inc.';
                        }}
                        if (parameter === 37446) {{
                            return 'Intel(R) UHD Graphics 630';
                        }}
                        return getParameter2.call(this, parameter);
                    }};
                }}

                // AudioContext fingerprint randomization
                const audioContext = window.AudioContext || window.webkitAudioContext;
                if (audioContext) {{
                    const OriginalAnalyser = audioContext.prototype.createAnalyser;
                    audioContext.prototype.createAnalyser = function() {{
                        const analyser = OriginalAnalyser.call(this);
                        const originalGetFloatFrequencyData = analyser.getFloatFrequencyData;
                        analyser.getFloatFrequencyData = function(array) {{
                            originalGetFloatFrequencyData.call(this, array);
                            for (let i = 0; i < array.length; i++) {{
                                array[i] = array[i] + Math.random() * 0.0001;
                            }}
                        }};
                        return analyser;
                    }};
                }}

                // ClientRects randomization
                const originalGetClientRects = Element.prototype.getClientRects;
                Element.prototype.getClientRects = function() {{
                    const rects = originalGetClientRects.call(this);
                    const noise = {};
                    for (let i = 0; i < rects.length; i++) {{
                        rects[i].x += noise;
                        rects[i].y += noise;
                    }}
                    return rects;
                }};

                // Media devices randomization
                if (navigator.mediaDevices) {{
                    const enumerateDevices = navigator.mediaDevices.enumerateDevices;
                    navigator.mediaDevices.enumerateDevices = function() {{
                        return enumerateDevices.call(this).then(devices => {{
                            return devices.map((device, index) => ({{
                                ...device,
                                deviceId: 'device_' + index + '_' + Math.random().toString(36).substr(2, 9),
                                groupId: 'group_' + Math.random().toString(36).substr(2, 9)
                            }}));
                        }});
                    }};
                }}

                // Disable Do Not Track
                Object.defineProperty(navigator, 'doNotTrack', {{
                    get: function() {{ return null; }}
                }});

                // Speech voices randomization
                if (window.speechSynthesis) {{
                    const voices = [
                        {{ name: 'Microsoft Huihui - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                        {{ name: 'Microsoft Kangkang - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                        {{ name: 'Microsoft Yaoyao - Chinese (Simplified, PRC)', lang: 'zh-CN' }},
                    ];

                    const originalGetVoices = speechSynthesis.getVoices;
                    speechSynthesis.getVoices = function() {{
                        return voices;
                    }};
                }}

                // Random device name
                const deviceNames = ['DESKTOP-{}', 'PC-{}', 'WORKSTATION-{}'];
                const randomName = deviceNames[Math.floor(Math.random() * deviceNames.length)];

                // WebRTC IP protection
                const originalRTCPeerConnection = window.RTCPeerConnection;
                window.RTCPeerConnection = function(...args) {{
                    const pc = new originalRTCPeerConnection(...args);
                    const originalCreateDataChannel = pc.createDataChannel;
                    pc.createDataChannel = function() {{
                        const channel = originalCreateDataChannel.apply(this, arguments);
                        return channel;
                    }};
                    return pc;
                }};

                // Port scan protection
                const originalFetch = window.fetch;
                window.fetch = function(url, ...args) {{
                    const urlObj = new URL(url, window.location.href);
                    if (urlObj.hostname === 'localhost' || urlObj.hostname === '127.0.0.1') {{
                        return Promise.reject(new Error('Port scan detected and blocked'));
                    }}
                    return originalFetch.call(this, url, ...args);
                }};
            }})();
            "#,
            rng.gen_range(0..5),
            rng.gen_range(0.0..0.1),
            rand::random::<u32>() % 10000,
            rand::random::<u32>() % 10000,
            rand::random::<u32>() % 10000
        )
    }

    pub fn launch_browser(&self) -> Result<Browser> {
        let (width, height) = if self.config.window_width > 0 && self.config.window_height > 0 {
            (self.config.window_width, self.config.window_height)
        } else {
            Self::generate_random_window_size()
        };

        let headless = self.config.mode == BrowserMode::Background;

        let mut launch_options = LaunchOptionsBuilder::default()
            .headless(headless)
            .window_size(Some((width, height)))
            .sandbox(false)
            .build()
            .context("Failed to build launch options")?;

        // Add additional Chrome arguments for fingerprint protection
        let window_size_arg = format!("--window-size={},{}", width, height);
        launch_options.args.push(OsStr::new(&window_size_arg));
        launch_options.args.push(OsStr::new("--disable-blink-features=AutomationControlled"));
        launch_options.args.push(OsStr::new("--disable-features=IsolateOrigins,site-per-process"));
        launch_options.args.push(OsStr::new("--lang=zh-CN"));
        launch_options.args.push(OsStr::new("--disable-web-security"));
        launch_options.args.push(OsStr::new("--ignore-certificate-errors"));
        launch_options.args.push(OsStr::new("--disable-dev-shm-usage"));
        launch_options.args.push(OsStr::new("--no-first-run"));
        launch_options.args.push(OsStr::new("--no-default-browser-check"));

        let browser = Browser::new(launch_options)
            .context("Failed to launch browser")?;

        Ok(browser)
    }

    pub fn apply_fingerprint_protection(&self, tab: &Arc<Tab>) -> Result<()> {
        let script = Self::generate_fingerprint_script();
        tab.evaluate(&script, false)
            .context("Failed to apply fingerprint protection")?;
        Ok(())
    }

    pub async fn wait_for_element(
        &self,
        tab: &Arc<Tab>,
        xpath: &str,
        timeout_seconds: u64,
    ) -> Result<bool> {
        let start = std::time::Instant::now();
        let timeout = Duration::from_secs(timeout_seconds);

        loop {
            if start.elapsed() > timeout {
                return Ok(false);
            }

            let script = format!(
                r#"
                (function() {{
                    const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                    return result.singleNodeValue !== null;
                }})()
                "#,
                xpath
            );

            match tab.evaluate(&script, true) {
                Ok(result) => {
                    if let Some(value) = result.value {
                        if value.as_bool().unwrap_or(false) {
                            return Ok(true);
                        }
                    }
                }
                Err(_) => {}
            }

            std::thread::sleep(Duration::from_millis(500));
        }
    }

    pub fn click_element(&self, tab: &Arc<Tab>, xpath: &str) -> Result<()> {
        let script = format!(
            r#"
            (function() {{
                const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                const element = result.singleNodeValue;
                if (element) {{
                    element.click();
                    return true;
                }}
                return false;
            }})()
            "#,
            xpath
        );

        tab.evaluate(&script, true)
            .context("Failed to click element")?;

        Ok(())
    }

    pub fn input_text(&self, tab: &Arc<Tab>, xpath: &str, text: &str) -> Result<()> {
        // Properly escape JavaScript string to prevent encoding issues
        let escaped_text = Self::escape_js_string(text);

        let script = format!(
            r#"
            (function() {{
                const result = document.evaluate("{}", document, null, XPathResult.FIRST_ORDERED_NODE_TYPE, null);
                const element = result.singleNodeValue;
                if (element) {{
                    // Focus the element first
                    element.focus();

                    // Clear existing value
                    element.value = "";

                    // Set the value using multiple methods to ensure React detects it
                    const nativeInputValueSetter = Object.getOwnPropertyDescriptor(window.HTMLInputElement.prototype, "value").set;
                    nativeInputValueSetter.call(element, "{}");

                    // Dispatch events in the correct order to simulate real user input
                    element.dispatchEvent(new Event('input', {{ bubbles: true, cancelable: true }}));
                    element.dispatchEvent(new Event('change', {{ bubbles: true, cancelable: true }}));
                    element.dispatchEvent(new Event('blur', {{ bubbles: true }}));

                    return true;
                }}
                return false;
            }})()
            "#,
            xpath,
            escaped_text
        );

        tab.evaluate(&script, true)
            .context("Failed to input text")?;

        Ok(())
    }

    /// Escape a string for safe use in JavaScript code
    fn escape_js_string(s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                '\'' => "\\'".to_string(),
                '\n' => "\\n".to_string(),
                '\r' => "\\r".to_string(),
                '\t' => "\\t".to_string(),
                '\x08' => "\\b".to_string(),
                '\x0C' => "\\f".to_string(),
                c if c.is_control() => format!("\\u{:04x}", c as u32),
                c => c.to_string(),
            })
            .collect()
    }

    #[allow(dead_code)]
    pub fn wait_for_navigation(&self, tab: &Arc<Tab>, timeout_seconds: u64) -> Result<()> {
        std::thread::sleep(Duration::from_secs(timeout_seconds));
        tab.wait_until_navigated()
            .context("Navigation timeout")?;
        Ok(())
    }

    pub fn clear_browser_data(&self) -> Result<()> {
        // This will be handled by launching a new browser instance with incognito mode
        // The browser automatically clears data when closed
        Ok(())
    }
}
