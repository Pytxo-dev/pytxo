import Link from "next/link";

import { LEGAL_CONTACT, LEGAL_UPDATED, LegalPage } from "@/components/site/legal-page";
import { PRO_INTERVAL, PRO_PRICE } from "@/lib/billing/pro";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata("/terms", "Terms · Pytxo", "Terms for Pytxo accounts, shared receipts and Pytxo Pro.");

export default function TermsPage() {
  return (
    <LegalPage title="Terms" updated={LEGAL_UPDATED}>
      <section>
        <h2>Who we are</h2>
        <p>
          Pytxo is made by MBCZ (&quot;we&quot;). Questions about these terms: <a href={`mailto:${LEGAL_CONTACT}`}>{LEGAL_CONTACT}</a>.
        </p>
      </section>
      <section>
        <h2>The software</h2>
        <p>
          The Pytxo CLI and Pytxo Desktop are open source under the MIT License, which governs their use and is provided
          without warranty. Everything they do on your machine works without an account and never depends on our hosted
          services.
        </p>
      </section>
      <section>
        <h2>Accounts</h2>
        <p>
          A Pytxo account is optional. Keep your sign-in secure and your details accurate. We may suspend an account that
          abuses the service, for example by trying to break or overload it, or by sharing unlawful content.
        </p>
      </section>
      <section>
        <h2>What you share</h2>
        <p>
          Shared receipts and Pybench results contain only what Pytxo sends when you choose to share: counts, outcomes,
          version numbers and, if you ask for them, file paths. A public receipt can be read by anyone with its link. You
          are responsible for what you share, and you let us host and display it to provide the service. You can delete a
          shared receipt from your account at any time.
        </p>
      </section>
      <section>
        <h2>Pytxo Pro</h2>
        <ul>
          <li>
            Pytxo Pro costs {PRO_PRICE} per {PRO_INTERVAL}, billed in advance. Taxes are shown at checkout.
          </li>
          <li>
            Payments are processed by Dodo Payments, our merchant of record and the seller of the subscription. Their
            terms also apply to your purchase.
          </li>
          <li>The subscription renews automatically each month until you cancel.</li>
          <li>
            Cancel any time from <Link href="/account">your account</Link> (Manage billing). Pro stays on until the end of
            the period you paid for.
          </li>
          <li>
            Refunds follow our <Link href="/refunds">refund policy</Link>.
          </li>
          <li>We will email subscribers at least 30 days before a price change applies to their next renewal.</li>
        </ul>
      </section>
      <section>
        <h2>Hosted services</h2>
        <p>
          Accounts, sharing, the Pybench board and Pytxo Pro are provided as is. We may change or end a hosted feature;
          if we end Pytxo Pro, we refund any unused, prepaid time.
        </p>
      </section>
      <section>
        <h2>Liability</h2>
        <p>
          To the extent the law allows, we are not liable for indirect or consequential losses, and our total liability
          is limited to what you paid us in the 12 months before the claim.
        </p>
      </section>
      <section>
        <h2>Changes</h2>
        <p>
          We may update these terms. The date above shows the latest version, and we email Pro subscribers about
          material changes.
        </p>
      </section>
    </LegalPage>
  );
}
