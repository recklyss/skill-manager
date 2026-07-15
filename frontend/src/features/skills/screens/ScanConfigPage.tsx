import { Link } from "react-router-dom";

import { PageHeader } from "../../../components/PageHeader";
import { ScanHarnessPicker } from "../components/scan/ScanHarnessPicker";
import { useSkillsCopy } from "../i18n";
import { useSkillScan } from "../model/use-skill-scan";
import { skillsRoutes } from "../public";

export default function ScanConfigPage() {
  const copy = useSkillsCopy();
  const scan = useSkillScan();

  return (
    <>
      <div className="page-chrome">
        <PageHeader title={copy.scan.configTitle} subtitle={copy.scan.configSubtitle} />
      </div>

      <section className="scan-config-page">
        <ScanHarnessPicker
          variant="list"
          harnesses={scan.harnesses}
          selectedHarness={scan.selectedHarness}
          harnessesLoaded={scan.harnessesLoaded}
          onSelectHarness={scan.selectHarness}
          idPrefix="scan-config"
        />

        <p className="scan-config-page__link">
          <Link to={skillsRoutes.inUse}>{copy.scan.runScansLink}</Link>
        </p>
      </section>
    </>
  );
}
