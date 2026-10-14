import Link from "next/link";

import { LEGAL_CONTACT, LEGAL_UPDATED, LegalPage } from "@/components/site/legal-page";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata("/refunds", "Refunds · Pytxo", "Refunds and cancellation for Pytxo Pro.");

export default function RefundsPage() {
  return (
    <LegalPage title="Refunds" updated={LEGAL_UPDATED}>
      <section>
        <h2>Cancel any time</h2>
        <p>
          Cancel Pytxo Pro from <Link href="/account">your account</Link> (Manage billing). You are not charged again,
          and Pro stays on until the end of the period you paid for.
        </p>
      </section>
      <section>
        <h2>Refunds</h2>
        <ul>
          <li>Your first Pro payment is fully refundable for 14 days, no questions asked.</li>
          <li>If you were charged by mistake, for example twice, we refund the extra charge.</li>
          <li>Other renewals are not refunded; cancelling stops the next one.</li>
        </ul>
        <p>
          To ask for a refund, email <a href={`mailto:${LEGAL_CONTACT}`}>{LEGAL_CONTACT}</a> from the address on your
          receipt. Refunds go back to the original payment method through Dodo Payments, our merchant of record.
        </p>
      </section>
    </LegalPage>
  );
}
