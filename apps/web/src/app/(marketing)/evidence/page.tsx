import { permanentRedirect } from "next/navigation";

// Evidence lives in the docs since 1.3, next to the method and limits of each record.
export default function EvidenceRedirect() {
  permanentRedirect("/docs/evidence");
}
