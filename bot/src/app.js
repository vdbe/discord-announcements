require('dotenv').config()
const GUILD_ID = process.env.GUILD_ID;
const CLIENT_ID = process.env.CLIENT_ID;
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

const { REST } = require('@discordjs/rest');
const { Routes } = require('discord-api-types/v9');

const commands = [
  {
    name: 'ping',
    description: 'Replies with Pong!',
  },
  {
    name: 'update',
    description: 'update',
  },
  {
    name: 'subscribe',
    description: 'subscribe to a feed',
    options: [{
      name: 'feed',
      description: 'url of the feed',
      required: true,
      type: 3,
    }]
  }
];

const rest = new REST({ version: '9' }).setToken(TOKEN);

(async () => {
  try {
    console.log('Started refreshing application (/) commands.');

    await rest.put(Routes.applicationGuildCommands(CLIENT_ID, GUILD_ID), { body: commands });

    console.log('Successfully reloaded application (/) commands.');
  } catch (error) {
    console.error(error);
  }
})();

const { Client, Intents, Interaction, MessageEmbed } = require('discord.js');
var client = new Client({ intents: [Intents.FLAGS.GUILDS] });

client.on('ready', () => {
  console.log(`Logged in as ${client.user.tag}!`);
});

client.on('interactionCreate', async (interaction) => {
  if (!interaction.isCommand()) return;

  if (interaction.commandName === 'ping') {
    await testCommand(interaction);
  } else if (interaction.commandName === 'update') {
    await updateCommand(interaction);
  } else if (interaction.commandName === 'subscribe') {
    await subscribeCommand(interaction);
  }
});

/**
* @param {Interaction} interaction
*/
async function subscribeCommand(interaction) {
  console.log(interaction);
  const guildid = interaction.guildId;
  const channelid = interaction.guildId;
  const feed = interaction.options.getString('feed');

  var client = new canvasrss_proto.CanvasRss(TARGET, grpc.credentials.createInsecure());

  let subscribeRequest = {
    guildId: guildid,
    channelId: channelid,
    feed: feed,
  }
  client.subscribe(subscribeRequest, function(err, response) {
    if (response.success === true) {
      interaction.reply(response.message);
    } else {
      interaction.reply({ content: response.message, ephemeral: true });
    }
  });
}

/**
* @param {Interaction} interaction
*/
async function updateCommand(interaction) {
  var client = new canvasrss_proto.CanvasRss(TARGET, grpc.credentials.createInsecure());

  let date = new Date(2022, 0, 1);
  let timestamp = Math.floor(date.getTime() / 1000);

  let newAnnouncementsRequest = {}
  let call = client.newAnnouncements(newAnnouncementsRequest);
  call.on('data', function(feed) {
    for (let key in feed.announcements) {
      const announcement = feed.announcements[key];
      const embed = buildAnnouncementEmbed(announcement);
      interaction.channel.send({ embeds: [embed] });
    }
  });

  call.on('end', function(feed) {
    interaction.reply({ content: 'Update complete', ephemeral: true });
    console.log("end");
  });
}

/**
* @param {Interaction} interaction
*/
async function testCommand(interaction) {
  var client = new canvasrss_proto.CanvasRss(Target, grpc.credentials.createInsecure());

  let date = new Date(2022, 0, 1);
  let timestamp = Math.floor(date.getTime() / 1000);
  console.log(timestamp);

  let listFeedsRequest = {
    after: {
      seconds: timestamp,
    },
  }
  console.log("Make updat request")
  let call = client.listFeeds(listFeedsRequest);
  call.on('data', function(feed) {
    for (let key in feed.announcements) {
      const announcement = feed.announcements[key];
      const embed = buildAnnouncementEmbed(announcement);
      console.log(embed);
      interaction.channel.send({ embeds: [embed] });
    }
  });

  call.on('end', function(feed) {
    console.log("end");
    interaction.reply({ content: 'test complete', ephemeral: true });
  });
}

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


function main() {
  let client = new canvasrss_proto.CanvasRss(TARGET, grpc.credentials.createInsecure());

  var user = 'world';

  client.sayHello({ name: user }, function(err, response) {
    console.log('Greeting:', response.message);
  });

  let date = new Date(2022, 0, 1);
  let timestamp = Math.floor(date.getTime() / 1000);
  console.log(timestamp);

  var listFeedsRequest = {
    after: {
      seconds: timestamp,
    },
  }
  var call = client.listFeeds(listFeedsRequest);
  call.on('data', function(feed) {
    for (let key in feed.announcements) {
      const announcement = feed.announcements[key];
      console.log(announcement.title);
      console.log(announcement.link);
      console.log(announcement.author);
      //console.log(announcement.content);
      console.log(announcement.published);
      let date = new Date(Date.UTC(1970, 0, 1)); // Epoch
      date.setSeconds(announcement.published.seconds);
      console.log(date);

      //const embed = new MessageEmbed()
      //  .setTitle(announcement.title)
      //  .setURL(announcement.link)
      //  .setAuthor(announcement.author)
      //  .setColor(0x00AE86)
      //  .setDescription(announcement.content)
      //  .setTimestamp();

      //interaction.channel.send({embeds: [embed]});
    }
  });

  call.on('end', function(feed) {
    console.log("end");
  });

}
//main();
client.login(TOKEN);
