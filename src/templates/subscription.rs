use mrml::mjml;
use mrmx::WithAttribute;
use mrmx_macros::view;

use crate::components::header;

pub(crate) fn subscription() -> mjml::Mjml {
    view! {
        <mjml>
        <mj-body>
          <mj-section>
            <mj-column>
              { header().into() }

              <mj-text font-size="20px" font-family="helvetica">
                <p>Hello Jade,</p>
                <p>"New edits have been added for artists that you've subscribed to!"</p>

                <h2>Changes for your subscribed artists</h2>
                <ul>
                  <li><a href="https://musicbrainz.org/artist/8d8d8a80-f74f-4f21-a44c-518cd6944ed2/edits">Nathan (English EDM artist)</a> (0 open, 1 applied)</li>
                </ul>

                <h2>More Information</h2>
                <ul>
                  <li><a href="https://musicbrainz.org/edit/subscribed?open=1">All open edits for your subscribed entities</a></li>
                  <li><a href="https://musicbrainz.org/edit/subscribed_editors?open=1">All open edits by your subscribed editors</a></li>
                </ul>
                <p>
                  This is a notification that edits have been added for artists, labels,
                  collections and editors to whom you subscribed on the MusicBrainz web site.
                  <a href="https://musicbrainz.org/user/Jellis16/subscriptions">Click here to view or edit your subscription list</a>.
                </p>
                <hr />
                  <p>Thanks for using MusicBrainz! <br />
      &mdash; The MusicBrainz Team</p>

                <p style="font-size: 0.8em;">Do not reply to this message. If you need help, please <a href="https://metabrainz.org/contact">contact us</a>.</p>

              </mj-text>

            </mj-column>
          </mj-section>
        </mj-body>
      </mjml>

    }
}
