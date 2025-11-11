// Script para funcionalidades do header 
(function () {
    const mainHeader = document.getElementById("main-header");
    let scrolled = false;

    function handleScroll() {
        const scrollTop = window.pageYOffset || document.documentElement.scrollTop;

        if (scrollTop > 50 && !scrolled) {
            scrolled = true;
            mainHeader.classList.add("header-scrolled");
        } else if (scrollTop <= 50 && scrolled) {
            scrolled = false;
            mainHeader.classList.remove("header-scrolled");
        }
    }

    // Adicionar listener de scroll
    window.addEventListener("scroll", handleScroll);
})();

// Mobile menu functionality
(function () {
    const mobileMenuOpen = document.getElementById("mobile-menu-open");
    const mobileMenuClose = document.getElementById("mobile-menu-close");
    const mobileMenuOverlay = document.getElementById("mobile-menu-overlay");
    const mobileMenuPanel = document.getElementById("mobile-menu-panel");

    function openMobileMenu() {
        mobileMenuOverlay.classList.add("show");
        mobileMenuPanel.classList.add("show");
        document.body.style.overflow = 'hidden';
    }

    function closeMobileMenu() {
        mobileMenuOverlay.classList.remove("show");
        mobileMenuPanel.classList.remove("show");
        document.body.style.overflow = '';
    }

    if (mobileMenuOpen && mobileMenuClose && mobileMenuOverlay && mobileMenuPanel) {
        mobileMenuOpen.addEventListener("click", openMobileMenu);
        mobileMenuClose.addEventListener("click", closeMobileMenu);

        // Close menu when clicking on overlay
        mobileMenuOverlay.addEventListener("click", (e) => {
            if (e.target === mobileMenuOverlay) {
                closeMobileMenu();
            }
        });

        // Close menu when clicking on links
        const mobileLinks = mobileMenuPanel.querySelectorAll('a[href^="#"]');
        mobileLinks.forEach((link) => {
            link.addEventListener("click", closeMobileMenu);
        });

        // Close menu on escape key
        document.addEventListener('keydown', (e) => {
            if (e.key === 'Escape') {
                closeMobileMenu();
            }
        });
    }
})();