import { Navigate, Route, Routes } from "react-router-dom";
import { ObsScoreboardPage } from "./obs-scoreboard/ObsScoreboardPage";

function App() {
  return (
    <Routes>
      <Route path="/" element={<ObsScoreboardPage />} />
      <Route path="/admin" element={<Navigate to="/" replace />} />
    </Routes>
  );
}

export default App;

