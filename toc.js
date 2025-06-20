// Populate the sidebar
//
// This is a script, and not included directly in the page, to control the total size of the book.
// The TOC contains an entry for each page, so if each page includes a copy of the TOC,
// the total size of the page becomes O(n**2).
class MDBookSidebarScrollbox extends HTMLElement {
    constructor() {
        super();
    }
    connectedCallback() {
        this.innerHTML = '<ol class="chapter"><li class="chapter-item expanded "><a href="chapter_1.html"><strong aria-hidden="true">1.</strong> Introduction</a></li><li class="chapter-item expanded "><a href="chapter_2.html"><strong aria-hidden="true">2.</strong> Installation</a></li><li class="chapter-item expanded affix "><li class="part-title">Architecture &amp; Bindings</li><li class="chapter-item expanded "><a href="architecture/ros2_overview.html"><strong aria-hidden="true">3.</strong> ROS 2 Architecture Overview</a></li><li class="chapter-item expanded "><a href="architecture/rcl_rmw_layers.html"><strong aria-hidden="true">4.</strong> RCL and RMW Layers</a></li><li class="chapter-item expanded "><a href="architecture/rust_ffi_bindings.html"><strong aria-hidden="true">5.</strong> Rust FFI Bindings</a></li><li class="chapter-item expanded "><a href="architecture/graph_context.html"><strong aria-hidden="true">6.</strong> Graph Context Implementation</a></li><li class="chapter-item expanded affix "><li class="part-title">Implementation Details</li><li class="chapter-item expanded "><a href="implementation/topic_info.html"><strong aria-hidden="true">7.</strong> Topic Information System</a></li><li class="chapter-item expanded "><a href="implementation/qos_profiles.html"><strong aria-hidden="true">8.</strong> QoS Profile Handling</a></li><li class="chapter-item expanded "><a href="implementation/endpoint_discovery.html"><strong aria-hidden="true">9.</strong> Endpoint Discovery</a></li><li class="chapter-item expanded "><a href="implementation/memory_management.html"><strong aria-hidden="true">10.</strong> Memory Management</a></li><li class="chapter-item expanded affix "><li class="part-title">Interface Definition Language (IDL)</li><li class="chapter-item expanded "><a href="idl/overview.html"><strong aria-hidden="true">11.</strong> IDL Tools Overview</a></li><li class="chapter-item expanded "><a href="idl/protobuf.html"><strong aria-hidden="true">12.</strong> Protobuf Integration</a></li><li class="chapter-item expanded "><a href="idl/ros2_messages.html"><strong aria-hidden="true">13.</strong> ROS2 Message System</a></li><li class="chapter-item expanded "><a href="idl/type_mapping.html"><strong aria-hidden="true">14.</strong> Type Mapping</a></li><li class="chapter-item expanded affix "><li class="part-title">Workspace Management</li><li class="chapter-item expanded "><a href="workspace/overview.html"><strong aria-hidden="true">15.</strong> Workspace Overview</a></li><li class="chapter-item expanded "><a href="workspace/build_system.html"><strong aria-hidden="true">16.</strong> Build System Architecture</a></li><li class="chapter-item expanded "><a href="workspace/package_discovery.html"><strong aria-hidden="true">17.</strong> Package Discovery</a></li><li class="chapter-item expanded "><a href="workspace/dependency_resolution.html"><strong aria-hidden="true">18.</strong> Dependency Resolution</a></li><li class="chapter-item expanded "><a href="workspace/environment_management.html"><strong aria-hidden="true">19.</strong> Environment Management</a></li><li class="chapter-item expanded "><a href="workspace/colcon_compatibility.html"><strong aria-hidden="true">20.</strong> Colcon Compatibility</a></li><li class="chapter-item expanded affix "><li class="part-title">Examples &amp; Usage</li><li class="chapter-item expanded "><a href="examples/basic_usage.html"><strong aria-hidden="true">21.</strong> Basic Usage</a></li><li class="chapter-item expanded "><a href="examples/advanced_usage.html"><strong aria-hidden="true">22.</strong> Advanced Usage</a></li><li class="chapter-item expanded "><a href="examples/integration_examples.html"><strong aria-hidden="true">23.</strong> Integration Examples</a></li><li class="chapter-item expanded "><a href="examples/command_reference.html"><strong aria-hidden="true">24.</strong> Command Reference</a></li></ol>';
        // Set the current, active page, and reveal it if it's hidden
        let current_page = document.location.href.toString().split("#")[0].split("?")[0];
        if (current_page.endsWith("/")) {
            current_page += "index.html";
        }
        var links = Array.prototype.slice.call(this.querySelectorAll("a"));
        var l = links.length;
        for (var i = 0; i < l; ++i) {
            var link = links[i];
            var href = link.getAttribute("href");
            if (href && !href.startsWith("#") && !/^(?:[a-z+]+:)?\/\//.test(href)) {
                link.href = path_to_root + href;
            }
            // The "index" page is supposed to alias the first chapter in the book.
            if (link.href === current_page || (i === 0 && path_to_root === "" && current_page.endsWith("/index.html"))) {
                link.classList.add("active");
                var parent = link.parentElement;
                if (parent && parent.classList.contains("chapter-item")) {
                    parent.classList.add("expanded");
                }
                while (parent) {
                    if (parent.tagName === "LI" && parent.previousElementSibling) {
                        if (parent.previousElementSibling.classList.contains("chapter-item")) {
                            parent.previousElementSibling.classList.add("expanded");
                        }
                    }
                    parent = parent.parentElement;
                }
            }
        }
        // Track and set sidebar scroll position
        this.addEventListener('click', function(e) {
            if (e.target.tagName === 'A') {
                sessionStorage.setItem('sidebar-scroll', this.scrollTop);
            }
        }, { passive: true });
        var sidebarScrollTop = sessionStorage.getItem('sidebar-scroll');
        sessionStorage.removeItem('sidebar-scroll');
        if (sidebarScrollTop) {
            // preserve sidebar scroll position when navigating via links within sidebar
            this.scrollTop = sidebarScrollTop;
        } else {
            // scroll sidebar to current active section when navigating via "next/previous chapter" buttons
            var activeSection = document.querySelector('#sidebar .active');
            if (activeSection) {
                activeSection.scrollIntoView({ block: 'center' });
            }
        }
        // Toggle buttons
        var sidebarAnchorToggles = document.querySelectorAll('#sidebar a.toggle');
        function toggleSection(ev) {
            ev.currentTarget.parentElement.classList.toggle('expanded');
        }
        Array.from(sidebarAnchorToggles).forEach(function (el) {
            el.addEventListener('click', toggleSection);
        });
    }
}
window.customElements.define("mdbook-sidebar-scrollbox", MDBookSidebarScrollbox);
