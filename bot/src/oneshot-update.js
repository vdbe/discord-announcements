require('dotenv').config()
const TOKEN = process.env.TOKEN;

const PROTO_PATH = __dirname + '/../../proto/canvasrss.proto';

var grpc = require('@grpc/grpc-js');
var protoLoader = require('@grpc/proto-loader');
var packageDefinition = protoLoader.loadSync(
  PROTO_PATH,
  {
    keepCase: true,
    longs: String,
    enums: String,
    defaults: true,
    oneofs: true
  });
var canvasrss_proto = grpc.loadPackageDefinition(packageDefinition).canvasrss;
const TARGET = 'localhost:50051';

const { Client, Intents, Interaction, MessageEmbed } = require('discord.js');


const TurndownService = require('turndown')
/**
* @param {Announcement} announcement
* @returns {MessageEmbed}
*/
function buildAnnouncementEmbed(announcement) {
  const ts = new TurndownService();

  let date = new Date(Date.UTC(1970, 0, 1)); // Epoch
  date.setSeconds(announcement.published.seconds);

  const embed = new MessageEmbed({
    color: '#E63F30',
    title: announcement.title,
    url: announcement.link,
    author: {
      name: announcement.author,
    },
    description: ts.turndown(announcement.content),
    footer: {
      text: date.toString(),
    }
  });
  return embed;
}

var bot = new Client({ intents: [Intents.FLAGS.GUILDS] });

bot.on('ready', () => {
  console.log(`Logged in as ${bot.user.tag}!`);

  (async () => {
    try {
      var client = new canvasrss_proto.CanvasRss(TARGET, grpc.credentials.createInsecure());

      let newAnnouncementsRequest = {}
      let call = client.newAnnouncements(newAnnouncementsRequest);

      let dataTasks = 0;
      let end = false;

      call.on('data', async function (feed) {
        dataTasks++;
        let channels = [];
        for (let key in feed.subscribers) {
          const subscriber = feed.subscribers[key];
          const serverId = subscriber.serverId;
          const channelId = subscriber.channelId

          const channel = await bot.channels.fetch(channelId);

          channels.push(channel);
        }

        for (let key in feed.announcements) {
          const announcement = feed.announcements[key];
          const embed = buildAnnouncementEmbed(announcement);

          for (let key in channels) {
            const channel = channels[key]
            channel.send({ embeds: [embed] });
          }
        }

        dataTasks--;
        if(end && dataTasks == 0) {
          bot.destroy();
        }
      });

      call.on('end', function (feed) {
        console.log("All data received");
        end = true;
        if(end && dataTasks == 0) {
          bot.destroy();
        }
      });

    } catch (error) {
      console.error(error);
    }
  })();
});

bot.login(TOKEN);
