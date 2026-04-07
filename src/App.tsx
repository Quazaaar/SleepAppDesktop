import { Routes, Route, useLocation } from "react-router-dom";
import { AnimatePresence, motion } from "framer-motion";
import Layout from "./components/Layout";
import Dashboard from "./pages/Dashboard";
import Timeline from "./pages/Timeline";
import Settings from "./pages/Settings";
import Apps from "./pages/Apps";
import Theme from "./pages/Theme";
import { useEscalationState } from "./hooks/useEscalationState";
import EscalationToastHandler from "./pages/overlays/EscalationToastHandler";
import PopupOverlay from "./pages/overlays/PopupOverlay";
import PanelOverlay from "./pages/overlays/PanelOverlay";
import FullscreenOverlay from "./pages/overlays/FullscreenOverlay";
import ResumeOverlay from "./pages/overlays/ResumeOverlay";

const pageVariants = {
  initial: { opacity: 0, y: 10 },
  animate: { opacity: 1, y: 0 },
  exit: { opacity: 0, y: -6 },
};

const pageTransition = { duration: 0.2, ease: "easeOut" as const };

function App() {
  const location = useLocation();
  const { level, message } = useEscalationState();

  return (
    <>
      {/* Escalation toast handler and window dispatcher.
          Rendered outside AnimatePresence so route changes cannot unmount it.
          Per STATE.md decision: overlay must render outside React Router. */}
      <EscalationToastHandler level={level} message={message} />

      {/* Overlay routes — these match when Tauri opens a new window at /#/overlay/*.
          They render standalone pages with no Layout wrapper.
          Placed outside AnimatePresence intentionally. */}
      <Routes>
        <Route path="/overlay/popup" element={<PopupOverlay />} />
        <Route path="/overlay/panel" element={<PanelOverlay />} />
        <Route path="/overlay/fullscreen" element={<FullscreenOverlay />} />
        <Route path="/overlay/resume" element={<ResumeOverlay />} />

        {/* Main app routes — all other paths get the full Layout + animated pages */}
        <Route
          path="/*"
          element={
            <Layout>
              <AnimatePresence mode="wait">
                <motion.div
                  key={location.pathname}
                  className="page-motion-wrapper"
                  variants={pageVariants}
                  initial="initial"
                  animate="animate"
                  exit="exit"
                  transition={pageTransition}
                >
                  <Routes location={location}>
                    <Route path="/" element={<Dashboard />} />
                    <Route path="/timeline" element={<Timeline />} />
                    <Route path="/settings" element={<Settings />} />
                    <Route path="/apps" element={<Apps />} />
                    <Route path="/theme" element={<Theme />} />
                  </Routes>
                </motion.div>
              </AnimatePresence>
            </Layout>
          }
        />
      </Routes>
    </>
  );
}

export default App;
