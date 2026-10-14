import { LEGAL_CONTACT, LEGAL_UPDATED, LegalPage } from "@/components/site/legal-page";
import { pageMetadata } from "@/lib/page-metadata";

export const metadata = pageMetadata("/privacy", "Privacy · Pytxo", "What Pytxo collects, why, and who processes it.");

export default function PrivacyPage() {
  return (
    <LegalPage title="Privacy" updated={LEGAL_UPDATED}>
      <section>
        <h2>On your machine</h2>
        <p>
          The Pytxo CLI and Desktop send no telemetry. Your repositories, commands, prompts and agent output stay on your
          machine. Data leaves it only when you use a hosted feature: signing in, sharing a receipt, submitting Pybench
          results, or the experimental Routing. Desktop also checks GitHub for new releases.
        </p>
      </section>
      <section>
        <h2>The website</h2>
        <p>
          pytxo.com uses no analytics or tracking cookies. Our host, Vercel, processes request data such as IP addresses
          to serve and protect the site.
        </p>
      </section>
      <section>
        <h2>Your account</h2>
        <ul>
          <li>Sign-in is handled by Clerk, which stores your email address, name if you give one, and sign-in records.</li>
          <li>
            Our account service (hosted on Railway) stores your account id, hashes of the sign-in credentials for your CLI
            and Desktop, the receipts and Pybench results you share, and whether each one is private.
          </li>
          <li>
            A shared receipt holds counts, the decision, agent names, versions and a digest, plus file paths only if you
            ask. A Pybench result holds its summary numbers, operating system, processor type and core count.
          </li>
        </ul>
      </section>
      <section>
        <h2>Payments</h2>
        <p>
          Dodo Payments processes Pytxo Pro payments as merchant of record and handles your payment details. We receive
          your subscription status and a customer id. We never see your card number.
        </p>
      </section>
      <section>
        <h2>Your choices</h2>
        <p>
          You can delete shared receipts from your account at any time. To delete your account and everything stored
          with it, email <a href={`mailto:${LEGAL_CONTACT}`}>{LEGAL_CONTACT}</a> from the address you sign in with. We do
          not sell personal data.
        </p>
      </section>
    </LegalPage>
  );
}
